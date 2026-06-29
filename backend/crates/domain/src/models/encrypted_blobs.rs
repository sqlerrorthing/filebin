use bytes::BytesMut;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Model {
    pub meta: super::encrypted_vault::Model,
    pub data: BytesMut
}
