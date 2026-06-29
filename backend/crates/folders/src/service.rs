pub mod basic;

use std::time::Duration;
use domain::models::{encrypted_blobs, folders};
use service::service;

#[service]
pub trait FoldersService {
    type Error;
    
    #[result]
    async fn delete_folder(&self, folder_id: folders::Id) -> bool;

    #[result]
    async fn rename_folder(&self, folder_id: folders::Id, encrypted_name: encrypted_blobs::Model) -> Option<folders::Model>;

    #[result]
    async fn find_folder_by_public_id(&self, public_id: folders::PublicId) -> Option<folders::Model>;

    #[result]
    async fn create_folder(&self, encrypted_name: encrypted_blobs::Model, expires: Option<Duration>) -> folders::Model;
}
