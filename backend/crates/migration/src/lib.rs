mod macros;
use crate::macros::migrator;
use sea_orm_migration::prelude::*;

migrator![m20260614_231015_initial];
