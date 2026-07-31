pub mod basic;

use std::error::Error;
use bytes::Bytes;
use futures::Stream;
use thiserror::Error;
use domain::models::{encrypted_blobs, encrypted_vault, files, folders};
use service::service;

#[derive(Debug, Error)]
pub enum UploadFileError<E> {
    #[error("the file is too large")]
    FileTooLarge,
    #[error("the folder is not found")]
    FolderNotFound,
    #[error("no enough permissions to upload")]
    NoPermissions,
    #[error("the folder is full")]
    FolderIsFull,
    #[error("stream error: {0}")]
    Stream(#[source] E),
}

#[service]
pub trait UploadService {
    type Error;

    #[result(UploadFileError<E>)]
    async fn upload_file_by_public_folder_id<E: Error>(
        &self,
        public_id: folders::PublicId,
        token: String,
        data_meta: encrypted_vault::Model,
        file_meta: encrypted_blobs::Model,
        chunks: impl Stream<Item = Result<Bytes, E>> + Send + 'static,
    ) -> files::Model
    where
        E: Send + 'static;
}
