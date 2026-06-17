use thiserror::Error;
use std::time::Duration;
use chrono::{Datelike, Timelike};
use sea_orm::prelude::DateTimeUtc;
use domain::entity;
use crate::schema::api::folder::v1::{Folder, FolderId, FolderToken};
use crate::schema::api::google;

impl From<entity::folders::PublicId> for FolderId {
    fn from(value: entity::folders::PublicId) -> Self {
        FolderId {
            value: value.into_inner().to_string(),
        }
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
                prost_types::Duration {
                    seconds: 0,
                    nanos: 0,
                },
            )),
        }
    }
}

impl From<&str> for FolderToken {
    fn from(value: &str) -> Self {
        FolderToken {
            value: value.to_owned()
        }
    }
}

impl From<entity::folders::Model> for Folder {
    fn from(value: entity::folders::Model) -> Self {
        Folder {
            id: value.public_id.into(),
            encrypted_name: value.encrypted_name,
            created_at: value.created_at.to_utc().into(),
            expired_at: value.expired_at.map(|exp| exp.to_utc().into()),
        }
    }
}

pub fn prost_duration_to_datetime_duration(duration: prost_types::Duration) -> chrono::Duration {
    chrono::Duration::seconds(duration.seconds)
    + chrono::Duration::nanoseconds(duration.nanos as _)
}

#[derive(Debug, Error)]
pub enum ConvertDurationError {
    #[error("the duration is negative")]
    Negative,
    #[error("out of range")]
    OutOfRange
}

pub fn prost_duration_to_std_duration(duration: prost_types::Duration) -> Result<Duration, ConvertDurationError> {
    if duration.seconds < 0 || duration.nanos < 0 {
        return Err(ConvertDurationError::Negative);
    }

    let secs = u64::try_from(duration.seconds)
        .map_err(|_| ConvertDurationError::OutOfRange)?;

    let nanos = u32::try_from(duration.nanos)
        .map_err(|_| ConvertDurationError::OutOfRange)?;

    Ok(Duration::new(secs, nanos))
}