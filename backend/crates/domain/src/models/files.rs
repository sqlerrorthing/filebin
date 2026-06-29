use chrono::{DateTime, Utc};
use nutype::nutype;
use serde::{Deserialize, Serialize};
use tinystr::TinyAsciiStr;
use sea_orm::entity::prelude::DeriveValueType;
use crate::macros::tiny_str_sea_orm_derive;
use uuid::Uuid;

#[nutype(
    const_fn,
    derive(
        Debug,
        PartialEq,
        Eq,
        Copy,
        Clone,
        Serialize,
        Deserialize,
        Hash,
        Deref,
        Display,
        FromStr
    ),
    derive_unchecked(DeriveValueType)
)]
pub struct Id(i32);

#[nutype(
    const_fn,
    derive(
        Debug,
        PartialEq,
        Eq,
        Clone,
        Serialize,
        Deserialize,
        Hash,
        Deref,
        Display,
        FromStr,
    )
)]
pub struct PublicId(TinyAsciiStr<16>);

#[nutype(
    const_fn,
    derive(
        Debug,
        PartialEq,
        Eq,
        Copy,
        Clone,
        Serialize,
        Deserialize,
        Hash,
        Deref,
        Display,
        FromStr
    ),
    derive_unchecked(DeriveValueType)
)]
pub struct StoragePath(Uuid);


#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Model {
    pub id: Id,
    pub public_id: PublicId,
    pub folder_id: super::folders::Id,
    pub data_meta: super::encrypted_vault::Model,
    pub meta: super::encrypted_blobs::Model,
    pub storage_path: StoragePath,
    pub file_size: i64,
    pub created_at: DateTime<Utc>
}
