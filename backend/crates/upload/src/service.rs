pub mod basic;

use std::error::Error;
use std::fmt::Display;
use std::ops::ControlFlow;
use std::str::FromStr;
use bytes::Bytes;
use futures::Stream;
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;
use domain::models::{encrypted_blobs, encrypted_vault, files, folders};
use service::service;

#[derive(Debug, Error)]
pub enum StreamUploadFileError<E> {
    #[error("the file is too large")]
    FileTooLarge,
    #[error("initiate upload error: {0}")]
    InitiateUploadError(#[from] InitiateUploadError),
    #[error("stream error: {0}")]
    Stream(#[source] E),
}

#[derive(Debug, Error)]
pub enum InitiateUploadError {
    #[error("the folder is not found")]
    FolderNotFound,
    #[error("no enough permissions to upload")]
    NoPermissions,
    #[error("the folder is full")]
    FolderIsFull,
}

#[derive(Debug, Error)]
pub enum ConsumeChunkError {
    #[error("the chunk is exceeds the limits")]
    Overflow,
    #[error("the file is too large")]
    FileTooLarge,
    #[error("the chunk is too large")]
    ChunkTooLarge,
    #[error("the upload is not found")]
    NotFound
}

#[service]
pub trait UploadService {
    type Error;
    type UploadId: Serialize + DeserializeOwned + FromStr + Display;

    #[result(StreamUploadFileError<E>)]
    async fn stream_upload_file_by_public_folder_id<E>(
        &self,
        public_id: folders::PublicId,
        token: String,
        data_meta: encrypted_vault::Model,
        file_meta: encrypted_blobs::Model,
        chunks: impl Stream<Item = Result<Bytes, E>> + Send + 'static,
    ) -> files::Model
    where
        E: Error + Send + 'static;

    /// Returns the upload id and max chunk size
    #[result(InitiateUploadError)]
    async fn initiate_upload(
        &self,
        public_id: folders::PublicId,
        token: String,
        data_meta: encrypted_vault::Model,
        file_meta: encrypted_blobs::Model
    ) -> (Self::UploadId, usize);

    #[result(ConsumeChunkError)]
    async fn consume_chunk(
        &self,
        upload_id: Self::UploadId,
        bytes: Bytes
    ) -> ControlFlow<files::Model>;
}
