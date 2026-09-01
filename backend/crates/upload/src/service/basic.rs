use crate::limited_stream::{LimitStreamError, LimitedStream};
use crate::service::{
    ConsumeChunkError, InitiateUploadError, StreamUploadFileError, UploadService,
};
use auth::service::TokenService;
use bytes::Bytes;
use derive_builder::Builder;
use derive_more::{Deref, DerefMut};
use derive_new::new;
use domain::models;
use domain::models::files::{Model, UploadFileData};
use domain::models::{encrypted_blobs, encrypted_vault};
use files::service::FilesService;
use files::storage::{
    HasRawMultipartUploadHandle, IntoRawMultipartUploadHandle, LockRawMultipartUploadHandle,
    RawMultipartUploadHandle,
};
use folders::service::FoldersService;
use futures::{Stream, StreamExt, TryStreamExt};
use nutype::nutype;
use serde::{Deserialize, Serialize};
use service::business;
use service::error::{OptionExt, ResultExt, ServiceError};
use std::borrow::Cow;
use std::marker::PhantomData;
use std::ops::{ControlFlow, Deref, DerefMut};
use storage::Storage;
use thiserror::Error;
use tokio::spawn;
use tracing::error;
use updates::service::UpdatesService;
use uuid::Uuid;

const MIN_BYTES_PER_SEC: usize = 10 * 1024;

#[derive(Builder, Debug, Clone)]
pub struct Limits {
    max_filesize: u64,
    max_files_per_folder: u32,
    max_chunck_size: usize,
}

#[derive(Debug, Clone, new)]
pub struct BasicUploadService<FilesS, FoldersS, TS, US, SS> {
    files_service: FilesS,
    folders_service: FoldersS,
    token_service: TS,
    updates_service: US,
    storage: SS,
    limits: Limits,
}

#[derive(Debug, Error)]
pub enum Error<FilesS: FilesService, FoldersS: FoldersService, TS: TokenService, SS: Storage> {
    #[error("files service error: {0}")]
    Files(#[source] FilesS::Error),
    #[error("folders service error: {0}")]
    Folders(#[source] FoldersS::Error),
    #[error("token service error: {0}")]
    Token(#[source] TS::Error),
    #[error("storage error: {0}")]
    Storage(#[source] SS::Error),
}

impl<FilesS, FoldersS, TS, US, SS> BasicUploadService<FilesS, FoldersS, TS, US, SS>
where
    FilesS: FilesService,
    FoldersS: FoldersService,
    TS: TokenService,
    US: UpdatesService,
    SS: Storage,
{
    async fn initiate_upload(
        &self,
        public_id: models::folders::PublicId,
        token: String,
    ) -> Result<
        models::folders::Model,
        ServiceError<InitiateUploadError, <Self as UploadService>::Error>,
    > {
        self.token_service
            .is_token_valid_for_folder(&public_id, token)
            .await
            .map_err(Error::Token)?
            .ok_or_business(InitiateUploadError::NoPermissions)?;

        let folder = self
            .folders_service
            .find_folder_by_public_id(public_id)
            .await
            .map_err(Error::Folders)?
            .ok_or_business(InitiateUploadError::FolderNotFound)?;

        self.files_service
            .files_count(folder.id)
            .await
            .map_err(Error::Files)?
            .lt(&(self.limits.max_files_per_folder as u64))
            .ok_or_business(InitiateUploadError::FolderIsFull)?;

        Ok(folder)
    }
}

impl<FilesS, FoldersS, TS, US, SS> BasicUploadService<FilesS, FoldersS, TS, US, SS> {
    fn upload_ttl(&self) -> u32 {
        (self.limits.max_chunck_size / MIN_BYTES_PER_SEC) as u32 * 2
    }
}

#[nutype(derive(Serialize, Deserialize, Display, Debug, Clone, FromStr))]
pub struct UploadId(Uuid);

#[derive(Serialize, Deserialize)]
struct ActiveUpload {
    bytes_received: i64,
    folder_id: models::folders::Id,
    data_meta: encrypted_vault::Model,
    file_meta: encrypted_blobs::Model,
}

impl From<ActiveUpload> for UploadFileData {
    fn from(value: ActiveUpload) -> Self {
        Self {
            file_size: value.bytes_received,
            meta: value.file_meta,
            data_meta: value.data_meta,
            folder_id: value.folder_id,
        }
    }
}

struct StorageSyncUploadHandle<S, R> {
    upload_id: UploadId,
    storage: S,
    ttl: u32,
    marker: PhantomData<fn() -> R>,
}

impl<S: Storage, R> Clone for StorageSyncUploadHandle<S, R> {
    fn clone(&self) -> Self {
        Self {
            upload_id: self.upload_id.clone(),
            storage: self.storage.clone(),
            ttl: self.ttl,
            marker: self.marker,
        }
    }
}

impl<S, R: RawMultipartUploadHandle> HasRawMultipartUploadHandle for StorageSyncUploadHandle<S, R> {
    type Raw = R;
}

impl<S: Storage, R: RawMultipartUploadHandle> IntoRawMultipartUploadHandle
    for StorageSyncUploadHandle<S, R>
{
    type Rest = Self;

    async fn into_raw(self) -> Option<(Self::Raw, Self::Rest)> {
        let handle: R = self.storage.get(handle_key(&self.upload_id)).await.ok()??;
        Some((handle, self))
    }

    fn from_raw(_: Self::Raw, rest: Self::Rest) -> Self {
        rest
    }
}

struct WriteGuardInner<S: Storage, R: RawMultipartUploadHandle> {
    inner: R,
    handle: StorageSyncUploadHandle<S, R>,
}

struct WriteGuard<'a, S: Storage, R: RawMultipartUploadHandle> {
    inner: Option<WriteGuardInner<S, R>>,
    marker: PhantomData<&'a ()>,
}

impl<S: Storage, R: RawMultipartUploadHandle> Deref for WriteGuard<'_, S, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.inner.as_ref().unwrap().inner
    }
}

impl<S: Storage, R: RawMultipartUploadHandle> DerefMut for WriteGuard<'_, S, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner.as_mut().unwrap().inner
    }
}

impl<S: Storage, R: RawMultipartUploadHandle> Drop for WriteGuard<'_, S, R> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            spawn(async move {
                if let Err(e) = inner
                    .handle
                    .storage
                    .set(
                        handle_key(&inner.handle.upload_id),
                        &inner.inner,
                        Some(inner.handle.ttl),
                    )
                    .await
                {
                    error!("Failed to update handle state: {e}");
                }
            });
        }
    }
}

impl<S: Storage, R: RawMultipartUploadHandle> LockRawMultipartUploadHandle
    for StorageSyncUploadHandle<S, R>
{
    type ReadRaw<'a> = Box<R>;
    type WriteRaw<'a> = WriteGuard<'a, S, R>;

    async fn read_raw<'a>(&'a self) -> Option<Self::ReadRaw<'a>> {
        let handle: R = self.storage.get(handle_key(&self.upload_id)).await.ok()??;
        Some(Box::new(handle))
    }

    async fn write_raw<'a>(&'a self) -> Option<Self::WriteRaw<'a>> {
        let handle: R = self.storage.get(handle_key(&self.upload_id)).await.ok()??;
        Some(WriteGuard {
            inner: Some(WriteGuardInner {
                inner: handle,
                handle: self.clone(),
            }),
            marker: PhantomData,
        })
    }
}

impl<FilesS, FoldersS, TS, US, SS> UploadService
    for BasicUploadService<FilesS, FoldersS, TS, US, SS>
where
    FilesS: FilesService,
    FoldersS: FoldersService,
    TS: TokenService,
    US: UpdatesService,
    SS: Storage,
{
    type Error = Error<FilesS, FoldersS, TS, SS>;
    type UploadId = UploadId;

    async fn stream_upload_file_by_public_folder_id<E: std::error::Error>(
        &self,
        public_id: models::folders::PublicId,
        token: String,
        data_meta: encrypted_vault::Model,
        file_meta: encrypted_blobs::Model,
        chunks: impl Stream<Item = Result<Bytes, E>> + Send + 'static,
    ) -> Result<models::files::Model, ServiceError<StreamUploadFileError<E>, Self::Error>>
    where
        E: Send + 'static,
    {
        let folder = self
            .initiate_upload(public_id, token)
            .await
            .map_business(Into::into)?;

        let res = self
            .files_service
            .upload_file(
                folder.id,
                data_meta,
                file_meta,
                LimitedStream::new(chunks, self.limits.max_filesize).map_err(|err| match err {
                    LimitStreamError::LimitExceeds => StreamUploadFileError::FileTooLarge,
                    LimitStreamError::Stream(s) => StreamUploadFileError::Stream(s),
                }),
            )
            .await
            .map_internal(Error::Files)?;

        self.updates_service.fire_file_uploaded(res.clone());
        Ok(res)
    }

    async fn initiate_upload(
        &self,
        public_id: models::folders::PublicId,
        token: String,
        data_meta: encrypted_vault::Model,
        file_meta: encrypted_blobs::Model,
    ) -> Result<(Self::UploadId, usize), ServiceError<InitiateUploadError, Self::Error>> {
        let folder = self.initiate_upload(public_id, token).await?;

        let handle = self
            .files_service
            .initiate_storage_upload()
            .await
            .map_err(Error::Files)?
            .into_raw()
            .await
            .expect("should be valid handle right after initialization")
            .0;

        let upload = ActiveUpload {
            bytes_received: 0,
            folder_id: folder.id,
            data_meta,
            file_meta,
        };

        let upload_id = UploadId::new(Uuid::now_v7());

        self.storage
            .set(upload_key(&upload_id), &upload, Some(self.upload_ttl()))
            .await
            .map_err(Error::Storage)?;

        self.storage
            .set(handle_key(&upload_id), &handle, Some(self.upload_ttl()))
            .await
            .map_err(Error::Storage)?;

        Ok((upload_id, self.limits.max_chunck_size))
    }

    async fn consume_chunk(
        &self,
        upload_id: Self::UploadId,
        bytes: Bytes,
    ) -> Result<ControlFlow<Model>, ServiceError<ConsumeChunkError, Self::Error>> {
        let len = bytes.len();

        if len > self.limits.max_chunck_size {
            return Err(business!(ConsumeChunkError::ChunkTooLarge));
        }

        let mut upload: ActiveUpload = self
            .storage
            .get(upload_key(&upload_id))
            .await
            .map_err(Error::Storage)?
            .ok_or_business(ConsumeChunkError::NotFound)?;

        let handle = StorageSyncUploadHandle {
            upload_id: upload_id.clone(),
            storage: self.storage.clone(),
            ttl: self.upload_ttl(),
            marker: PhantomData,
        };

        if len as i64 + upload.bytes_received >= self.limits.max_filesize as _ {
            _ = self
                .storage
                .bulk_delete([upload_key(&upload_id), handle_key(&upload_id)])
                .await;

            return Err(business!(ConsumeChunkError::FileTooLarge));
        }

        upload.bytes_received += len as i64;

        self.storage.set(upload_key(&upload_id), &upload, Some(self.upload_ttl()))
            .await
            .map_err(Error::Storage)?;

        if !bytes.is_empty() {
            self.files_service.upload_file_chunk(bytes, handle.clone()).await
                .map_err(Error::Files)?;
        }

        if len == self.limits.max_chunck_size {
            return Ok(ControlFlow::Continue(()))
        }

        let file = self.files_service.complete_multipart_upload(
            upload.into(),
            handle
        ).await.map_err(Error::Files)?;

        _ = self
            .storage
            .bulk_delete([upload_key(&upload_id), handle_key(&upload_id)])
            .await;

        Ok(ControlFlow::Break(file))
    }
}

fn upload_key(upload_id: &UploadId) -> String {
    format!("uploading:{upload_id}")
}

fn handle_key(upload_id: &UploadId) -> String {
    format!("uploading:{upload_id}:handle")
}
