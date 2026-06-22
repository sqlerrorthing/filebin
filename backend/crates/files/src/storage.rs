use bytes::Bytes;
use futures_core::Stream;
use domain::entity::files;
use service::service;

pub mod s3;

/// Contains files prefix.
/// Should be used with `{FILES_PREFIX}/{key}`
pub const FILES_PREFIX: &str = "files";

#[service]
pub trait FilesStorage: 'static {
    type Error;

    type MultipartUploadHandle;

    type GetFileStream: Stream<Item = Result<Bytes, Self::Error>>;
    
    /// Creates new mulipart upload stream
    ///
    /// Returns the upload handler, when it drops its automatically calls [`FilesStorage::abort_multipart_upload`]
    #[result]
    async fn create_multipart_upload(&self, key: files::StoragePath) -> Self::MultipartUploadHandle;

    #[result]
    async fn upload_part(&self, handle: &Self::MultipartUploadHandle, part: Bytes);

    /// Completes the multipart upload
    ///
    /// Returns the key
    #[result]
    async fn complete_multipart_upload(&self, handle: Self::MultipartUploadHandle) -> files::StoragePath;

    /// Bulk deletes the provided ids
    #[result]
    async fn bulk_delete(&self, ids: Vec<files::StoragePath>);
    
    #[result]
    async fn get_file(&self, path: files::StoragePath) -> Option<Self::GetFileStream>;
}
