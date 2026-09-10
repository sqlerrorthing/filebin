use chrono::{DateTime, Utc};
use domain_macros::Model;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Model)]
#[model(
    newtypes(
        Id(i64),
        PublicId(tinystr::TinyAsciiStr<8>),

        #[derive(FromJsonQueryResult)]
        FolderName(super::super::encrypted_blobs::Model)
    ),
    inputs(
        NewFolder(
            public_id,
            ..FolderName as encrypted_name,
            expired_at
        ),
    )
)]
pub struct Model {
    pub id: Id,
    pub public_id: PublicId,
    pub encrypted_name: FolderName,
    pub expired_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
