pub mod basic;

use std::time::Duration;
use domain::entity::folders;
use service::service;

#[service]
pub trait FoldersService: 'static {
    type Error;

    #[result]
    async fn find_folder_by_public_id(&self, public_id: folders::PublicId) -> Option<folders::Model>;

    #[result]
    async fn create_folder(&self, encrypted_name: String, expires: Option<Duration>) -> folders::Model;
}
