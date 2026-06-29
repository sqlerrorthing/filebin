use crate::macros::tiny_str_sea_orm_derive;
use chrono::{DateTime, Utc};
use nutype::nutype;
use sea_orm::entity::prelude::DeriveValueType;
use serde::{Deserialize, Serialize};
use tinystr::TinyAsciiStr;

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
        FromStr
    )
)]
pub struct PublicId(TinyAsciiStr<8>);

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Model {
    pub id: Id,
    pub public_id: PublicId,
    pub encrypted_name: super::encrypted_blobs::Model,
    pub expired_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
