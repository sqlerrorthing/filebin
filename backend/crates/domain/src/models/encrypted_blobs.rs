use bytes::{Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use domain_macros::Model;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Model)]
pub struct Model {
    pub meta: super::encrypted_vault::Model,
    pub data: Bytes
}
