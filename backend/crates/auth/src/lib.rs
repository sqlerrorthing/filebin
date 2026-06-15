use std::str::FromStr;
use domain::entity::folders;
use service::service;

#[service]
pub trait TokenService {
    type Token: ToString + FromStr;
    
    async fn generate_token_for_folder_long_id(
        folder_long_id: folders::LongId
    ) -> Self::Token;
    
    async fn is_token_valid_for_folder(
        folder_long_id: folders::LongId,
        token: Self::Token
    ) -> bool;
}
