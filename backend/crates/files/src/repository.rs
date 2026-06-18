pub mod db;

use domain::entity::{files, folders};
use service::service;

#[service]
pub trait FilesRepository: 'static {
    type Error;

    /// Deletes all files from the folder, returning deleted files
    #[result]
    async fn delete_files_from_folder(&self, folder_id: folders::Id) -> Vec<files::Model>;

    #[result]
    async fn insert(&self, folder: files::ActiveModel) -> files::Model;
    
    #[result]
    async fn update(&self, folder: files::ActiveModel) -> files::Model;
}
