use std::error::Error;
use std::str::FromStr;
use crate::schema::ServiceErrorExt;
use crate::schema::api::folder::v1::folder_update::Update;
use crate::schema::api::folder::v1::{Algorithm, EncryptedBlobs, EncryptedVault, FileDeleted, FileId, FileMetadata, FileView, Folder, FolderId, FolderName, FolderNameChanged, FolderToken, NewFile, Version};
use crate::schema::api::google;
use chrono::{Datelike, Timelike};
use domain::models;
use pbjson_types::Empty;
use sea_orm::prelude::DateTimeUtc;
use std::time::Duration;
use thiserror::Error;
use tinystr::{TinyStr8, TinyStr16};
use tonic::Status;

impl From<models::folders::PublicId> for FolderId {
    fn from(value: models::folders::PublicId) -> Self {
        FolderId {
            value: value.into_inner().to_string(),
        }
    }
}

impl From<&updates::service::FolderUpdateKind> for Update {
    fn from(update: &updates::service::FolderUpdateKind) -> Self {
        use updates::service::FolderUpdateKind;

        match update {
            FolderUpdateKind::FileUploaded { file } => Update::NewFile(NewFile {
                file: file.clone().into(),
            }),
            FolderUpdateKind::FolderRenamed { new_folder_name } => {
                Update::FolderNameChanged(FolderNameChanged {
                    name: new_folder_name.clone().into_inner().into(),
                })
            }
            FolderUpdateKind::FolderDeleted { .. } => Update::FolderDeleted(Empty {}),
            FolderUpdateKind::FileDeleted { file } => Update::FileDeleted(FileDeleted {
                file_id: file.public_id.clone().into(),
            }),
        }
    }
}

impl From<models::files::PublicId> for FileId {
    fn from(value: models::files::PublicId) -> Self {
        FileId {
            value: value.into_inner().to_string(),
        }
    }
}

impl TryFrom<FolderId> for models::folders::PublicId {
    type Error = Status;

    fn try_from(value: FolderId) -> Result<Self, Self::Error> {
        Ok(models::folders::PublicId::new(
            TinyStr8::try_from_str(&value.value).ok_or_invalid_argument("invalid id")?,
        ))
    }
}

impl TryFrom<FileId> for models::files::PublicId {
    type Error = Status;

    fn try_from(value: FileId) -> Result<Self, Self::Error> {
        Ok(models::files::PublicId::new(
            TinyStr16::try_from_str(&value.value).ok_or_invalid_argument("invalid id")?,
        ))
    }
}

impl From<DateTimeUtc> for google::r#type::DateTime {
    fn from(value: DateTimeUtc) -> Self {
        google::r#type::DateTime {
            year: value.year(),
            month: value.month() as i32,
            day: value.day() as i32,
            hours: value.hour() as i32,
            minutes: value.minute() as i32,
            seconds: value.second() as i32,
            nanos: value.nanosecond() as i32,
            time_offset: Some(google::r#type::date_time::TimeOffset::UtcOffset(
                pbjson_types::Duration {
                    seconds: 0,
                    nanos: 0,
                },
            )),
        }
    }
}

impl From<String> for FolderToken {
    fn from(value: String) -> Self {
        FolderToken { value }
    }
}

impl From<models::encrypted_blobs::Model> for FolderName {
    fn from(value: models::encrypted_blobs::Model) -> Self {
        Self {
            value: value.into()
        }
    }
}

impl From<models::folders::Model> for Folder {
    fn from(value: models::folders::Model) -> Self {
        Folder {
            id: value.public_id.into(),
            name: value.encrypted_name.into_inner().into(),
            created_at: value.created_at.to_utc().into(),
            expired_at: value.expired_at.map(|exp| exp.to_utc().into()),
        }
    }
}

impl From<models::encrypted_blobs::Model> for EncryptedBlobs {
    fn from(value: models::encrypted_blobs::Model) -> Self {
        Self {
            meta: value.meta.into(),
            data: value.data,
        }
    }
}

impl TryFrom<EncryptedVault> for models::encrypted_vault::Model {
    type Error = Status;
    
    fn try_from(value: EncryptedVault) -> Result<Self, Self::Error> {
        Ok(Self {
            iv: value.iv.parse().map_err(|_| Status::invalid_argument("invalid iv"))?,
            tag: value.tag.parse().map_err(|_| Status::invalid_argument("invalid iv"))?,
            ver: models::encrypted_vault::Version::new(value.version.value as _),
            algo: value.algo.value.parse().map_err(|_| Status::invalid_argument("unsupported algo"))?,
        })
    }
}

impl TryFrom<EncryptedBlobs> for models::encrypted_blobs::Model {
    type Error = Status;
    
    fn try_from(value: EncryptedBlobs) -> Result<Self, Self::Error> {
        Ok(Self {
            meta: value.meta.try_into()?,
            data: value.data,
        })
    }
}

impl From<models::encrypted_vault::Version> for Version {
    fn from(value: models::encrypted_vault::Version) -> Self {
        Self { value: *value as _ }
    }
}

impl From<models::encrypted_vault::EncryptionAlgo> for Algorithm {
    fn from(value: models::encrypted_vault::EncryptionAlgo) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

impl From<models::encrypted_vault::Model> for EncryptedVault {
    fn from(value: models::encrypted_vault::Model) -> Self {
        Self {
            iv: value.iv.to_string(),
            tag: value.tag.to_string(),
            version: value.ver.into(),
            algo: value.algo.into(),
        }
    }
}

impl From<models::files::Model> for FileView {
    fn from(value: models::files::Model) -> Self {
        FileView {
            id: value.public_id.into(),
            metadata: FileMetadata {
                value: value.meta.into(),
            },
            size: value.file_size,
        }
    }
}

pub fn prost_duration_to_datetime_duration(duration: prost_types::Duration) -> chrono::Duration {
    chrono::Duration::seconds(duration.seconds) + chrono::Duration::nanoseconds(duration.nanos as _)
}

#[derive(Debug, Error)]
pub enum ConvertDurationError {
    #[error("the duration is negative")]
    Negative,
    #[error("out of range")]
    OutOfRange,
}

pub fn prost_duration_to_std_duration(
    duration: pbjson_types::Duration,
) -> Result<Duration, ConvertDurationError> {
    if duration.seconds < 0 || duration.nanos < 0 {
        return Err(ConvertDurationError::Negative);
    }

    let secs = u64::try_from(duration.seconds).map_err(|_| ConvertDurationError::OutOfRange)?;

    let nanos = u32::try_from(duration.nanos).map_err(|_| ConvertDurationError::OutOfRange)?;

    Ok(Duration::new(secs, nanos))
}

pub trait FromStrExt: FromStr
where
    <Self as FromStr>::Err: Error
{
    fn from_str_or_invalid_argument<'a>(s: &str, value_name: impl Into<Option<&'a str>>) -> Result<Self, Status> {
        let mut msg = "invalid value".to_string();

        if let Some(value_name) = value_name.into() {
            msg.push_str(&format!(": {value_name}"));
        }

        Self::from_str(s)
            .ok_or_invalid_argument(msg)
    }
}

impl<T: FromStr> FromStrExt for T
where
    <Self as FromStr>::Err: Error
{}