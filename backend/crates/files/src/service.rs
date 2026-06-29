pub mod basic;

use std::fmt::Debug;
use bytes::Bytes;
use domain::models::{files, folders};
use futures_core::Stream;
use service::service;
use thiserror::Error;

#[service]
pub trait FilesService {
    type Error;
    type GetFileStream: Stream<Item = Result<Bytes, Self::Error>> + Debug;

    fn min_upload_chunk_size(&self) -> i64;

    #[result]
    async fn files_count(&self, folder_id: folders::Id) -> u64;

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
    #[result(E)]
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
    
    #[result]
    async fn delete_file(&self, file_id: files::Id) -> Option<files::Model>;
    
    #[result]
    async fn delete_file_from_folder_by_public_id(&self, folder_id: folders::Id, public_id: files::PublicId) -> Option<files::Model>;
}
