use chrono::{DateTime, Utc};
use domain_macros::Model;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Model)]
#[model(
    newtypes(
        Id(i32),
        PublicId(tinystr::TinyAsciiStr<16>),
        StoragePath(Uuid)
    ),
    inputs(
        NewFile(public_id, folder_id, data_meta, meta, storage_path, file_size)
    )
)]
pub struct Model {
    pub id: Id,
    pub public_id: PublicId,
    pub folder_id: super::folders::Id,
    pub data_meta: super::encrypted_vault::Model,
    pub meta: super::encrypted_blobs::Model,
    pub storage_path: StoragePath,
    pub file_size: i64,
    pub created_at: DateTime<Utc>,
}
