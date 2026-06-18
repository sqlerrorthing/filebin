use domain::entity::{files, folders};
use service::service;

pub mod db;

#[service]
pub trait FoldersRepository: 'static {
    type Error;

    #[result]
    async fn find_folder_by_public_id(&self, folder: folders::PublicId) -> Option<folders::Model>;

    #[result]
    async fn insert(&self, folder: folders::ActiveModel) -> folders::Model;

    #[result]
    async fn update(&self, folder: folders::ActiveModel) -> folders::Model;

    #[result]
    async fn delete(&self, folder_id: folders::Id) -> bool;
}
