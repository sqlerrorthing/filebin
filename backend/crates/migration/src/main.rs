use sea_orm_migration::prelude::*;
use crate::migrator::migrator;

mod migrator;

migrator! [
    m20260614_231015_initial
];

#[tokio::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
