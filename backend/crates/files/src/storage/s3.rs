use crate::storage::{FILES_PREFIX, FilesStorage};
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::complete_multipart_upload::CompleteMultipartUploadError;
use aws_sdk_s3::operation::create_multipart_upload::CreateMultipartUploadError;
use aws_sdk_s3::operation::upload_part::UploadPartError;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart, Delete, ObjectIdentifier};
use aws_sdk_s3::{Client as S3Client, Client};
use aws_smithy_types::error::operation::BuildError;
use bytes::Bytes;
use derive_new::new;
use domain::entity::files;
use domain::sync::shared_string::SharedString;
use std::hint::cold_path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, Ordering};
use aws_sdk_s3::operation::delete_objects::DeleteObjectsError;
use thiserror::Error;
use tokio::spawn;
use tracing::error;

const AWS_BULK_DELETE_CHUNKS: usize = 1000;

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

    #[error("the multipart handler dropped")]
    MultipartHandlerDropped,
}

#[derive(Debug)]
pub struct S3MultipartUpload {
    client: Client,
    bucket: SharedString,
    key: SharedString,
    upload_id: SharedString,
    next_part_number: AtomicI32,
    completed_parts: Mutex<Vec<CompletedPart>>,
}

#[derive(Debug)]
pub struct S3MultipartUploadHandle {
    inner: Option<S3MultipartUpload>,
}

impl Drop for S3MultipartUploadHandle {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            cold_path();
            spawn(async move {
                if let Err(e) = inner
                    .client
                    .abort_multipart_upload()
                    .bucket(inner.bucket)
                    .key(inner.key)
                    .upload_id(inner.upload_id)
                    .send()
                    .await
                {
                    error!("Abort multipart upload caught an error: {e}")
                }
            });
        }
    }
}

impl FilesStorage for S3FilesStorage {
    type Error = Error;
    type MultipartUploadHandle = S3MultipartUploadHandle;

    async fn create_multipart_upload(
        &self,
        key: String,
    ) -> Result<Self::MultipartUploadHandle, Self::Error> {
        let res = self
            .client
            .create_multipart_upload()
            .bucket(self.bucket.clone())
            .key(&key)
            .send()
            .await?;

        let upload_id = res.upload_id.ok_or(Error::MissingUploadId)?;

        Ok(S3MultipartUploadHandle {
            inner: Some(S3MultipartUpload {
                client: self.client.clone(),
                bucket: self.bucket.clone(),
                key: key.into(),
                upload_id: upload_id.into(),
                next_part_number: AtomicI32::new(1),
                completed_parts: Default::default(),
            }),
        })
    }

    async fn upload_part(
        &self,
        handle: &Self::MultipartUploadHandle,
        part: Bytes,
    ) -> Result<(), Self::Error> {
        let handle = handle
            .inner
            .as_ref()
            .ok_or(Error::MultipartHandlerDropped)?;

        let part_number = handle.next_part_number.fetch_add(1, Ordering::SeqCst);

        let res = self
            .client
            .upload_part()
            .bucket(handle.bucket.clone())
            .key(handle.key.clone())
            .upload_id(handle.upload_id.clone())
            .part_number(part_number)
            .body(part.into())
            .send()
            .await?;

        let completed_part = CompletedPart::builder()
            .e_tag(res.e_tag.unwrap_or_default())
            .part_number(part_number)
            .build();

        let mut parts = handle.completed_parts.lock().unwrap();
        parts.push(completed_part);

        Ok(())
    }

    async fn complete_multipart_upload(
        &self,
        mut handle: Self::MultipartUploadHandle,
    ) -> Result<String, Self::Error> {
        let handle_inner = handle.inner.take().ok_or(Error::MultipartHandlerDropped)?;
        let mut parts = handle_inner.completed_parts.lock().unwrap().clone();
        parts.sort_by_key(|p| p.part_number);

        if let Err(e) = self
            .client
            .complete_multipart_upload()
            .bucket(handle_inner.bucket.clone())
            .key(handle_inner.key.clone())
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
            handle.inner = Some(handle_inner);
            return Err(e.into());
        }

        Ok(handle_inner.key.into())
    }

    async fn bulk_delete(&self, ids: Vec<files::StoragePath>) -> Result<(), Self::Error> {
        if ids.is_empty() {
            return Ok(());
        }

        for chunk in ids.chunks(AWS_BULK_DELETE_CHUNKS) {
            let object_ids = chunk
                .into_iter()
                .map(|key| {
                    ObjectIdentifier::builder()
                        .key(format!("{FILES_PREFIX}/{key}"))
                        .build()
                })
                .collect::<Result<Vec<_>, _>>()?;

            self.client
                .delete_objects()
                .bucket(self.bucket.clone())
                .delete(Delete::builder()
                    .set_objects(Some(object_ids))
                    .build()?
                )
                .send()
                .await?;
        }

        Ok(())
    }
}
