pub mod basic;

use std::time::Duration;
use thiserror::Error;
use domain::models::{encrypted_blobs, folders};
use service::service;

#[derive(Debug, Error)]
pub enum RenameFolderError {
    #[error("folder name is empty")]
    Empty,
    #[error("folder name is too long")]
    TooLong
}

#[service]
pub trait FoldersService {
    type Error;
    
    #[result]
    async fn delete_folder(&self, folder_id: folders::Id) -> bool;

    #[result(RenameFolderError)]
    async fn rename_folder(&self, folder_id: folders::Id, new_name: folders::FolderName) -> Option<folders::Model>;

    #[result]
    async fn find_folder_by_public_id(&self, public_id: folders::PublicId) -> Option<folders::Model>;

    #[result]
    async fn create_folder(&self, encrypted_name: folders::FolderName, expires: Option<Duration>) -> folders::Model;
}
