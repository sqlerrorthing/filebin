use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use domain_macros::Model;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Model)]
#[model(
    newtypes(
        Id(i32),
        PublicId(tinystr::TinyAsciiStr<8>)
    ),
    inputs(
        NewFolder(public_id, encrypted_name, expired_at)
    )
)]
pub struct Model {
    pub id: Id,
    pub public_id: PublicId,
    pub encrypted_name: super::encrypted_blobs::Model,
    pub expired_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
