#![feature(min_specialization)]

use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use crate::config::{CONFIG, Db, Storage};
use crate::schema::api::folder::v1::folder_service_server::FolderServiceServer;
use crate::v1::folder::BasicGrpcFolderService;
use auth::service::jwt::JwtTokenService;
use folders::service::basic::BasicFoldersService;
use id_generator::service::random::RandomIdGeneratorService;
use sea_orm::{Database, DatabaseConnection, DbErr};
use sea_orm_migration::migrator::MigratorTrait;
use secrecy::ExposeSecret;
use tonic::transport::Server;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};
use files::service::basic::BasicFilesService;
use files::storage::s3::S3FilesStorage;

pub mod config;
pub mod schema;
pub mod v1;

async fn up_db(config: &Db) -> Result<DatabaseConnection, DbErr> {
    info!("Connecting to postgres db");
    let db = Database::connect(&config.postgres_url).await?;

    info!("Db connected. Running migrator");
    migration::Migrator::up(&db, None).await?;

    Ok(db)
}

async fn up_s3_client(config: &Storage) -> aws_sdk_s3::Client {
    let creds = Credentials::new(
        config.access_key.expose_secret(),
        config.secret_key.expose_secret(),
        None,
        None,
        "config"
    );

    let mut builder = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(config.region.clone()))
        .credentials_provider(creds);

    if let Some(url) = &config.endpoint_url {
        builder = builder.endpoint_url(url);
    }

    let loaded = builder.load().await;
    let s3_config_builder = aws_sdk_s3::config::Builder::from(&loaded)
        .force_path_style(config.force_path_style.unwrap_or(true));

    aws_sdk_s3::Client::from_conf(s3_config_builder.build())
}

fn tracing() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init()
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing();

    let db = up_db(&CONFIG.db).await?;

    let token_service = JwtTokenService::new(
        chrono::Duration::from_std(CONFIG.jwt.expires)?,
        CONFIG.jwt.secret.expose_secret(),
    );

    let files_storage = S3FilesStorage::new(
        up_s3_client(&CONFIG.storage).await,
        CONFIG.storage.bucket.clone().into()
    );

    let files_service = BasicFilesService::new(
        files_storage,
        db.clone(),
        RandomIdGeneratorService,
        CONFIG.limits.max_filesize.as_u64()
    );

    let folders_service = BasicFoldersService::new(db, RandomIdGeneratorService);

    Server::builder()
        .add_service(FolderServiceServer::new(BasicGrpcFolderService::new(
            files_service,
            folders_service,
            token_service,
        )))
        .serve("[::1]:50051".parse()?)
        .await?;

    Ok(())
}
