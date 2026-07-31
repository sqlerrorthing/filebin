use bytes::Bytes;
use sea_orm::FromJsonQueryResult;
use domain_macros::Model;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Model, FromJsonQueryResult)]
#[model()]
pub struct Model {
    pub meta: super::encrypted_vault::Model,
    pub data: Bytes,
}
