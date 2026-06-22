pub mod basic;

use bytes::Bytes;
use domain::entity::{files, folders};
use futures_core::Stream;
use service::error::ServiceError;
use service::service;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UploadFileError<E> {
    #[error("the file is too large")]
    FileTooLarge,
    #[error("stream error: {0}")]
    Stream(#[source] E),
}

#[service]
pub trait FilesService: 'static {
    type Error;
    type GetFileStream: Stream<Item = Result<Bytes, Self::Error>>;

    fn min_upload_chunk_size(&self) -> i64;

    #[result]
    async fn delete_files_from_folder(&self, folder_id: folders::Id);

    #[result]
    async fn list_folder_files(&self, folder_id: folders::Id) -> Vec<files::Model>;

    #[result]
    async fn find_file_by_public_id_in_folder_by_id(
        &self,
        folder_id: folders::Id,
        public_id: files::PublicId,
    ) -> Option<files::Model>;

    /// Uploads the file to folder.
    /// To cancel the upload, just drop the future or send [`Err`] through the chunks stream
    #[result(UploadFileError<E>)]
    async fn upload_file<E>(
        &self,
        folder_id: folders::Id,
        encrypted_path: String,
        encrypted_mime_type: String,
        encrypted_file_hash: String,
        chunks: impl Stream<Item = Result<Bytes, E>> + Send + 'static,
    ) -> files::Model
    where
        E: Send + 'static;

    #[result]
    async fn get_file_by_storage_path(
        &self,
        storage_path: files::StoragePath
    ) -> Option<Self::GetFileStream>;
}
