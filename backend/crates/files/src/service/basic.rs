use crate::repository::FilesRepository;
use crate::service::FilesService;
use crate::storage::{FilesStorage, HasRawMultipartUploadHandle, IntoRawMultipartUploadHandle};
use bytes::Bytes;
use derive_new::new;
use domain::models::files::{NewFile, UploadFileData};
use domain::models::{encrypted_blobs, encrypted_vault, files, folders};
use futures_core::Stream;
use futures_util::{StreamExt, TryStreamExt};
use id_generator::service::IdGeneratorService;
use service::business;
use service::error::ServiceError;
use std::fmt::Debug;
use thiserror::Error;
use tracing::{Level, error, span};
use updates::service::UpdatesService;

#[derive(Clone, Debug, new)]
pub struct BasicFilesService<FS, FR, IGS, US> {
    files_storage: FS,
    files_repository: FR,
    id_generator_service: IGS,
    updates_service: US,
}

#[derive(Debug, Error)]
pub enum Error<FS: FilesStorage, FR: FilesRepository> {
    #[error("files storage error: {0}")]
    Storage(#[source] FS::Error),
    #[error("files repository error: {0}")]
    Repository(#[source] FR::Error),
}

impl<FS, FR, IGS, US> FilesService for BasicFilesService<FS, FR, IGS, US>
where
    FS: FilesStorage,
    FR: FilesRepository,
    IGS: IdGeneratorService,
    US: UpdatesService,
{
    type Error = Error<FS, FR>;
    type GetFileStream = impl Stream<Item = Result<Bytes, Self::Error>> + Debug;
    type MultipartUploadHandle = FS::MultipartUploadHandle;

    fn min_upload_chunk_size(&self) -> i64 {
        5 * 1024 * 1024
    }

    async fn files_count(&self, folder_id: folders::Id) -> Result<u64, Self::Error> {
        self.files_repository
            .files_count(folder_id)
            .await
            .map_err(Error::Repository)
    }

    async fn delete_files_from_folder(&self, folder_id: folders::Id) -> Result<(), Self::Error> {
        let files = self
            .files_repository
            .delete_files_from_folder(folder_id)
            .await
            .map_err(Error::Repository)?;

        if files.is_empty() {
            return Ok(());
        }

        self.files_storage
            .bulk_delete(files.into_iter().map(|f| f.storage_path).collect())
            .await
            .map_err(Error::Storage)
    }

    async fn list_folder_files(
        &self,
        folder_id: folders::Id,
    ) -> Result<Vec<files::Model>, Self::Error> {
        self.files_repository
            .list_folder_files(folder_id)
            .await
            .map_err(Error::Repository)
    }

    async fn find_file_by_public_id_in_folder_by_id(
        &self,
        folder_id: folders::Id,
        public_id: files::PublicId,
    ) -> Result<Option<files::Model>, Self::Error> {
        Ok(self
            .files_repository
            .find_file_by_public_id(public_id)
            .await
            .map_err(Error::Repository)?
            .take_if(|f| f.folder_id == folder_id))
    }

    async fn upload_file<E>(
        &self,
        folder_id: folders::Id,
        data_meta: encrypted_vault::Model,
        file_meta: encrypted_blobs::Model,
        chunks: impl Stream<Item = Result<Bytes, E>> + Send + 'static,
    ) -> Result<files::Model, ServiceError<E, Self::Error>>
    where
        E: Send + 'static,
    {
        let handle = self.initiate_storage_upload().await?;
        tokio::pin!(chunks);

        let mut total_bytes_received = 0_u64;

        loop {
            let chunk = match chunks.next().await {
                Some(Ok(c)) => c,
                Some(Err(e)) => return Err(business!(e)),
                None => break,
            };

            total_bytes_received += chunk.len() as u64;

            self.files_storage
                .upload_part(&handle, chunk)
                .await
                .map_err(Error::Storage)?;
        }

        let storage_path = self
            .files_storage
            .complete_multipart_upload(handle)
            .await
            .map_err(Error::Storage)?;

        let file = self
            .files_repository
            .new_file(NewFile {
                public_id: self.id_generator_service.next_public_file_id(),
                folder_id,
                data_meta,
                meta: file_meta,
                storage_path,
                file_size: total_bytes_received as _,
            })
            .await
            .map_err(Error::Repository)?;

        Ok(file)
    }

    async fn initiate_storage_upload(&self) -> Result<Self::MultipartUploadHandle, Self::Error> {
        self.files_storage
            .create_multipart_upload(self.id_generator_service.next_file_storage_path())
            .await
            .map_err(Error::Storage)
    }

    async fn upload_file_chunk<H>(&self, chunk: Bytes, handle: H) -> Result<(), Self::Error>
    where
        H: IntoRawMultipartUploadHandle<
            Raw = <Self::MultipartUploadHandle as HasRawMultipartUploadHandle>::Raw,
        >,
    {
        self.files_storage
            .upload_part(&handle, chunk)
            .await
            .map_err(Error::Storage)
    }

    async fn complete_multipart_upload<H>(
        &self,
        upload_file: UploadFileData,
        handle: H,
    ) -> Result<files::Model, Self::Error>
    where
        H: IntoRawMultipartUploadHandle<
            Raw = <Self::MultipartUploadHandle as HasRawMultipartUploadHandle>::Raw,
        >,
    {
        let storage_path = self
            .files_storage
            .complete_multipart_upload(handle)
            .await
            .map_err(Error::Storage)?;

        let file = self
            .files_repository
            .new_file(NewFile {
                public_id: self.id_generator_service.next_public_file_id(),
                folder_id: upload_file.folder_id,
                data_meta: upload_file.data_meta,
                meta: upload_file.meta,
                storage_path,
                file_size: upload_file.file_size,
            })
            .await
            .map_err(Error::Repository)?;

        Ok(file)
    }

    async fn get_file_by_storage_path(
        &self,
        storage_path: files::StoragePath,
    ) -> Result<Option<Self::GetFileStream>, Self::Error> {
        Ok(self
            .files_storage
            .get_file(storage_path)
            .await
            .map_err(Error::Storage)?
            .map(|stream| stream.map_err(Error::Storage)))
    }

    async fn delete_file(&self, file_id: files::Id) -> Result<Option<files::Model>, Self::Error> {
        let _span = span!(Level::DEBUG, "delete file", file_id = %file_id);

        let deleted = self
            .files_repository
            .delete_file(file_id)
            .await
            .map_err(Error::Repository)?;

        if let Some(deleted) = &deleted {
            if let Err(e) = self.files_storage.delete(deleted.storage_path).await {
                error!(error = %e, "failed to delete file from storage!")
            }

            self.updates_service.fire_file_deleted(deleted.clone());
        }

        Ok(deleted)
    }

    async fn delete_file_from_folder_by_public_id(
        &self,
        folder_id: folders::Id,
        public_id: files::PublicId,
    ) -> Result<Option<files::Model>, Self::Error> {
        if let Some(file) = self
            .files_repository
            .find_file_by_public_id(public_id)
            .await
            .map_err(Error::Repository)?
            && file.folder_id == folder_id
        {
            self.delete_file(file.id).await
        } else {
            Ok(None)
        }
    }
}
