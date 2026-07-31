use bytes::Bytes;
use sea_orm::FromJsonQueryResult;
use domain_macros::Model;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::base64::Base64;

#[serde_as]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Model, FromJsonQueryResult)]
#[model()]
pub struct Model {
    pub meta: super::encrypted_vault::Model,
    #[serde_as(as = "Base64")]
    pub data: Bytes,
}
