use bytes::{Bytes, BytesMut};
use domain_macros::Model;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Model)]
#[model(
    newtypes(
        Id(i32)
    ),
    inputs(
        NewBlob(
            ..super::encrypted_vault::NewVault,
            data
        ),
    )
)]
pub struct Model {
    pub id: Id,
    pub meta: super::encrypted_vault::Model,
    pub data: Bytes,
}
