use domain::entity::folders;
use service::service;

pub mod db;

#[service]
pub trait FoldersRepository: 'static {
    type Error;

    #[result]
    async fn insert(&self, folder: folders::ActiveModel) -> folders::Model;

    #[result]
    async fn update(&self, folder: folders::ActiveModel) -> folders::Model;
}
