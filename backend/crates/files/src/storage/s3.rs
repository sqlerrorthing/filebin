use crate::storage::{
    LockRawMultipartUploadHandle, FILES_PREFIX, FilesStorage, HasRawMultipartUploadHandle,
    IntoRawMultipartUploadHandle,
};
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::complete_multipart_upload::CompleteMultipartUploadError;
use aws_sdk_s3::operation::create_multipart_upload::CreateMultipartUploadError;
use aws_sdk_s3::operation::delete_object::DeleteObjectError;
use aws_sdk_s3::operation::delete_objects::DeleteObjectsError;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::operation::upload_part::UploadPartError;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart, Delete, ObjectIdentifier};
use aws_sdk_s3::{Client as S3Client, Client};
use aws_smithy_types::byte_stream;
use aws_smithy_types::error::operation::BuildError;
use bytes::Bytes;
use derive_new::new;
use domain::models::files;
use domain::sync::shared_string::SharedString;
use futures_core::Stream;
use futures_util::{TryStreamExt, stream};
use parking_lot::lock_api::{RwLockReadGuard, RwLockWriteGuard};
use parking_lot::{RawRwLock, RwLock};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hint::cold_path;
use std::sync::atomic::AtomicI32;
use thiserror::Error;
use tokio::spawn;
use tracing::error;

const AWS_BULK_DELETE_CHUNKS: usize = 150;

#[derive(Debug, Clone, new)]
pub struct S3FilesStorage {
    client: S3Client,
    bucket: SharedString,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("create multipart upload error: {0}")]
    CreateMultipartUpload(#[from] SdkError<CreateMultipartUploadError>),

    #[error("upload multipart part error: {0}")]
    UploadPart(#[from] SdkError<UploadPartError>),

    #[error("complete multipart upload error: {0}")]
    CompleteMultipartUpload(#[from] SdkError<CompleteMultipartUploadError>),

    #[error("missing `uplodad_id` after create multipart upload")]
    MissingUploadId,

    #[error("build error: {0}")]
    Build(#[from] BuildError),

    #[error("delete objects error: {0}")]
    DeleteObjects(#[from] SdkError<DeleteObjectsError>),

    #[error("delete object error: {0}")]
    DeleteObject(#[from] SdkError<DeleteObjectError>),

    #[error("get object error: {0}")]
    GetObject(#[from] SdkError<GetObjectError>),

    #[error("the multipart handler dropped")]
    MultipartHandlerDropped,

    #[error("steram error: {0}")]
    Stream(#[from] byte_stream::error::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawCompletedPart {
    part_number: i32,
    e_tag: String,
}

impl From<RawCompletedPart> for CompletedPart {
    fn from(value: RawCompletedPart) -> Self {
        CompletedPart::builder()
            .part_number(value.part_number)
            .e_tag(value.e_tag)
            .build()
    }
}

impl From<CompletedPart> for RawCompletedPart {
    fn from(p: CompletedPart) -> Self {
        Self {
            part_number: p.part_number().unwrap_or_default(),
            e_tag: p.e_tag().map(ToString::to_string).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawS3MultipartFileUpload {
    bucket: SharedString,
    key: files::StoragePath,
    upload_id: SharedString,
    next_part_number: i32,
    completed_parts: Vec<RawCompletedPart>,
}

#[derive(Debug)]
pub struct S3MultipartFileUploadInner {
    client: Client,
    inner: RwLock<RawS3MultipartFileUpload>,
}

#[derive(Debug)]
pub struct S3MultipartUploadHandle {
    inner: Option<S3MultipartFileUploadInner>,
}

impl Drop for S3MultipartUploadHandle {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            cold_path();

            let handle = inner.inner.into_inner();
            spawn(async move {
                if let Err(e) = inner
                    .client
                    .abort_multipart_upload()
                    .bucket(handle.bucket)
                    .key(format!("{FILES_PREFIX}/{}", handle.key))
                    .upload_id(handle.upload_id)
                    .send()
                    .await
                {
                    error!("Abort multipart upload caught an error: {e}")
                }
            });
        }
    }
}

impl HasRawMultipartUploadHandle for S3MultipartUploadHandle {
    type Raw = RawS3MultipartFileUpload;
}

impl LockRawMultipartUploadHandle for S3MultipartUploadHandle {
    type ReadRaw<'a> = RwLockReadGuard<'a, RawRwLock, Self::Raw>;
    type WriteRaw<'a> = RwLockWriteGuard<'a, RawRwLock, Self::Raw>;

    async fn read_raw(&self) -> Option<Self::ReadRaw<'_>> {
        self.inner.as_ref().map(|i| i.inner.read())
    }

    async fn write_raw(&self) -> Option<Self::WriteRaw<'_>> {
        self.inner.as_ref().map(|i| i.inner.write())
    }
}

impl IntoRawMultipartUploadHandle for S3MultipartUploadHandle {
    type Rest = Client;

    async fn into_raw(mut self) -> Option<(Self::Raw, Self::Rest)> {
        let inner = self.inner.take()?;
        Some((inner.inner.into_inner(), inner.client))
    }

    fn from_raw(raw: Self::Raw, rest: Self::Rest) -> Self {
        Self {
            inner: Some(S3MultipartFileUploadInner {
                client: rest,
                inner: RwLock::new(raw),
            }),
        }
    }
}

impl FilesStorage for S3FilesStorage {
    type Error = Error;
    type MultipartUploadHandle = S3MultipartUploadHandle;
    type GetFileStream = impl Stream<Item = Result<Bytes, Self::Error>> + Debug;

    async fn create_multipart_upload(
        &self,
        key: files::StoragePath,
    ) -> Result<Self::MultipartUploadHandle, Self::Error> {
        let res = self
            .client
            .create_multipart_upload()
            .bucket(self.bucket.clone())
            .key(format!("{FILES_PREFIX}/{key}"))
            .send()
            .await?;

        let upload_id = res.upload_id.ok_or(Error::MissingUploadId)?;

        Ok(S3MultipartUploadHandle {
            inner: Some(S3MultipartFileUploadInner {
                client: self.client.clone(),
                inner: RwLock::new(RawS3MultipartFileUpload {
                    bucket: self.bucket.clone(),
                    key,
                    upload_id: upload_id.into(),
                    next_part_number: 1,
                    completed_parts: Default::default(),
                }),
            }),
        })
    }

    async fn upload_part(
        &self,
        handle: &impl LockRawMultipartUploadHandle<
            Raw = <Self::MultipartUploadHandle as HasRawMultipartUploadHandle>::Raw,
        >,
        part: Bytes,
    ) -> Result<(), Self::Error> {
        let (part_number, bucket, upload_id, key) = handle
            .with_write(|h| {
                let part = h.next_part_number;
                h.next_part_number += 1;

                (part, h.bucket.clone(), h.upload_id.clone(), h.key.clone())
            })
            .await
            .ok_or(Error::MultipartHandlerDropped)?;

        let res = self
            .client
            .upload_part()
            .bucket(bucket.clone())
            .key(format!("{FILES_PREFIX}/{}", key))
            .upload_id(upload_id.clone())
            .part_number(part_number)
            .body(part.into())
            .send()
            .await?;

        let completed_part = RawCompletedPart {
            e_tag: res.e_tag.unwrap_or_default(),
            part_number,
        };

        handle.with_write(|h| h.completed_parts.push(completed_part))
            .await
            .ok_or(Error::MultipartHandlerDropped)?;

        Ok(())
    }

    async fn complete_multipart_upload<H>(
        &self,
        handle: H,
    ) -> Result<files::StoragePath, Self::Error>
    where
        H: IntoRawMultipartUploadHandle<
            Raw = <Self::MultipartUploadHandle as HasRawMultipartUploadHandle>::Raw,
        >
    {
        let (handle_inner, rest) = handle.into_raw().await.ok_or(Error::MultipartHandlerDropped)?;
        let mut parts = handle_inner.completed_parts.clone();
        parts.sort_by_key(|p| p.part_number);
        let parts = parts.into_iter().map(Into::into).collect();

        if let Err(e) = self
            .client
            .complete_multipart_upload()
            .bucket(handle_inner.bucket.clone())
            .key(format!("{FILES_PREFIX}/{}", handle_inner.key))
            .upload_id(handle_inner.upload_id.clone())
            .multipart_upload(
                CompletedMultipartUpload::builder()
                    .set_parts(Some(parts))
                    .build(),
            )
            .send()
            .await
        {
            cold_path();
            H::from_raw(handle_inner, rest); // trigger RAII cleanup
            return Err(e.into());
        }

        Ok(handle_inner.key)
    }

    async fn bulk_delete(&self, ids: Vec<files::StoragePath>) -> Result<(), Self::Error> {
        if ids.is_empty() {
            return Ok(());
        }

        for chunk in ids.chunks(AWS_BULK_DELETE_CHUNKS) {
            let object_ids = chunk
                .iter()
                .map(|key| {
                    ObjectIdentifier::builder()
                        .key(format!("{FILES_PREFIX}/{key}"))
                        .build()
                })
                .collect::<Result<Vec<_>, _>>()?;

            self.client
                .delete_objects()
                .bucket(self.bucket.clone())
                .delete(Delete::builder().set_objects(Some(object_ids)).build()?)
                .send()
                .await?;
        }

        Ok(())
    }

    async fn delete(&self, id: files::StoragePath) -> Result<(), Self::Error> {
        self.client
            .delete_object()
            .key(format!("{FILES_PREFIX}/{id}"))
            .bucket(self.bucket.clone())
            .send()
            .await?;

        Ok(())
    }

    async fn get_file(
        &self,
        path: files::StoragePath,
    ) -> Result<Option<Self::GetFileStream>, Self::Error> {
        let object = self
            .client
            .get_object()
            .bucket(self.bucket.clone())
            .key(format!("{FILES_PREFIX}/{path}"))
            .send()
            .await;

        let object = match object {
            Err(SdkError::ServiceError(e)) if e.err().is_no_such_key() => {
                return Ok(None);
            }
            x => x?,
        };

        let mut stream = Box::pin(object.body);

        Ok(Some(
            stream::poll_fn(move |cx| stream.as_mut().poll_next(cx)).map_err(Into::into),
        ))
    }
}
