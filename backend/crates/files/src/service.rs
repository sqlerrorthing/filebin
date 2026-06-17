pub mod basic;

use bytes::Bytes;
use domain::entity::{files, folders};
use service::error::ServiceError;
use service::service;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Error)]
pub enum UploadFileError {
    #[error("the file is too large")]
    FileTooLarge,
    #[error("upload cancelled")]
    Cancelled
}

#[service]
pub trait FilesService: 'static {
    type Error;

    fn min_upload_chunk_size(&self) -> i64;

    /// Uploads the file to folder.
    /// To cancel the upload, just drop the [`oneshot::Receiver`] half or call cancellation
    fn upload_file(
        &self,
        folder_id: folders::Id,
        encrypted_path: String,
        encrypted_mime_type: String,
        encrypted_file_hash: String,
        chunks: mpsc::Receiver<Bytes>,
        cancellation: CancellationToken
    ) -> oneshot::Receiver<Result<files::Model, ServiceError<UploadFileError, Self::Error>>>;
}
