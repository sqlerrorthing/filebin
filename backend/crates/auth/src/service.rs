use std::error::Error;
use domain::entity::folders;
use service::service;

pub mod jwt;

#[service]
pub trait TokenService {
    type Error: Error;

    #[result]
    async fn generate_token_for_folder_public_id(
        &self,
        folder_long_id: &folders::PublicId
    ) -> String;

    #[result]
    async fn is_token_valid_for_folder(
        &self,
        folder_long_id: &folders::PublicId,
        token: String
    ) -> bool;
}
