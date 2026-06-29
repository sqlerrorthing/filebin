use domain::models::{encrypted_blobs, folders};
use service::service;

pub mod db;
pub mod cache;

#[service]
pub trait FoldersRepository {
    type Error;

    #[result]
    async fn find_folder_by_public_id(&self, public_id: folders::PublicId) -> Option<folders::Model>;

    #[result]
    async fn new_folder(&self, create_folder: folders::NewFolder) -> folders::Model;

    #[result]
    async fn delete(&self, folder_id: folders::Id) -> Option<folders::Model>;
    
    #[result]
    async fn rename(&self, folder_id: folders::Id, encrypted_name: encrypted_blobs::Model) -> Option<folders::Model>;
}
