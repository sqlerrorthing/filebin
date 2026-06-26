pub mod db;
pub mod cache;

use domain::entity::{files, folders};
use service::service;

#[service]
pub trait FilesRepository {
    type Error;

    #[result]
    async fn files_count(&self, folder_id: folders::Id) -> u64;
    
    /// Deletes all files from the folder, returning deleted files
    #[result]
    async fn delete_files_from_folder(&self, folder_id: folders::Id) -> Vec<files::Model>;

    #[result]
    async fn find_file_by_public_id(&self, public_id: files::PublicId) -> Option<files::Model>;
    
    #[result]
    async fn list_folder_files(&self, folder_id: folders::Id) -> Vec<files::Model>;
    
    #[result]
    async fn insert(&self, files: files::ActiveModel) -> files::Model;
    
    #[result]
    async fn update(&self, files: files::ActiveModel) -> files::Model;
}
