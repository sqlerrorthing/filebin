use std::error::Error;
use domain::entity::folders;
use service::service;

#[service]
pub trait FolderService {
    type Error: Error;

    #[result]
    async fn create_folder() -> folders::Entity;
}
