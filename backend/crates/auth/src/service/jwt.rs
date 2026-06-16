use crate::service::TokenService;
use chrono::{DateTime, Duration, Utc};
use domain::entity::folders::PublicId;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
struct ModifyFolderToken {
    #[serde(rename = "exp", with = "chrono::serde::ts_seconds")]
    expiration: DateTime<Utc>,

    #[serde(rename = "iat", with = "chrono::serde::ts_seconds")]
    issued_at: DateTime<Utc>,

    folder: PublicId,
}

#[derive(Debug)]
pub struct JwtTokenService {
    expires: Duration,
    encoding: EncodingKey,
    decoding: DecodingKey,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("invalid expiration timestamp")]
    InvalidExpires,
}

impl TokenService for JwtTokenService {
    type Error = Error;

    async fn generate_token_for_folder_long_id(
        &self,
        folder_long_id: PublicId,
    ) -> Result<String, Self::Error> {
        let claims = ModifyFolderToken {
            expiration: Utc::now()
                .checked_add_signed(self.expires)
                .ok_or(Error::InvalidExpires)?,
            issued_at: Utc::now(),
            folder: folder_long_id,
        };

        encode(&Header::default(), &claims, &self.encoding).map_err(Into::into)
    }

    async fn is_token_valid_for_folder(
        &self,
        folder_long_id: PublicId,
        token: String,
    ) -> Result<bool, Self::Error> {
        Ok(
            decode::<ModifyFolderToken>(&token, &self.decoding, &Validation::new(Algorithm::HS256))
                .map_or(false, |data| data.claims.folder == folder_long_id),
        )
    }
}
