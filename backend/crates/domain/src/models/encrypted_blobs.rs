use bytes::Bytes;
use domain_macros::Model;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::serde_as;

#[serde_as]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Model, FromJsonQueryResult)]
#[model()]
pub struct Model {
    pub meta: super::encrypted_vault::Model,
    #[serde_as(as = "Base64")]
    pub data: Bytes,
}
