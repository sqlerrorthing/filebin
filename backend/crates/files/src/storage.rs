use bytes::Bytes;
use service::service;

pub mod s3;

/// Contains files prefix.
/// Should be used with `{FILES_PREFIX}/{key}`
pub const FILES_PREFIX: &str = "files";

#[service]
pub trait FilesStorage: 'static {
    type Error;

    type MultipartUploadHandle;

    /// Creates new mulipart upload stream
    ///
    /// Returns the upload handler, when it drops its automatically calls [`FilesStorage::abort_multipart_upload`]
    #[result]
    async fn create_multipart_upload(&self, key: String) -> Self::MultipartUploadHandle;

    #[result]
    async fn upload_part(&self, handle: &Self::MultipartUploadHandle, part: Bytes);

    /// Completes the multipart upload
    ///
    /// Returns the key
    #[result]
    async fn complete_multipart_upload(&self, handle: Self::MultipartUploadHandle) -> String;
}
