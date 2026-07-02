use bytes::{Bytes, BytesMut};
use domain_macros::Model;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Model)]
#[model(
    inputs(
        NewBlob(
            ..super::encrypted_vault::NewVault,
            data
        ),
    )
)]
pub struct Model {
    pub meta: super::encrypted_vault::Model,
    pub data: Bytes,
}
