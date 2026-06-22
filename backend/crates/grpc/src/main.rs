#![feature(min_specialization, impl_trait_in_assoc_type)]

use crate::config::{CONFIG, Db, Redis, Storage};
use crate::schema::api::folder::v1::files_service_server::FilesServiceServer;
use crate::schema::api::folder::v1::folder_service_server::FolderServiceServer;
use crate::v1::files::BasicGrpcFilesService;
use crate::v1::folder::BasicGrpcFolderService;
use auth::service::jwt::JwtTokenService;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use cache::Cache;
use deadpool_redis::{CreatePoolError, Runtime};
use download::service::basic::BasicDownloadService;
use files::service::basic::BasicFilesService;
use files::storage::s3::S3FilesStorage;
use folders::service::basic::BasicFoldersService;
use id_generator::service::random::RandomIdGeneratorService;
use sea_orm::{Database, DatabaseConnection, DbErr};
use sea_orm_migration::migrator::MigratorTrait;
use secrecy::ExposeSecret;
use std::any::type_name_of_val;
use std::time::Duration;
use tokio::time::sleep;
use tonic::transport::Server;
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

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

async fn up_redis(config: &Redis) -> Result<deadpool_redis::Pool, deadpool_redis::PoolError> {
    loop {
        let cfg = deadpool_redis::Config::from_url(&config.url);
        let builder = cfg.builder().unwrap().runtime(Runtime::Tokio1);

        let error = match builder.build().map_err(CreatePoolError::Build) {
            Ok(pool) => match pool.get().await {
                Ok(_) => break Ok(pool),
                Err(e) => e.to_string(),
            },
            Err(e) => e.to_string(),
        };

        error!("Failed connect to redis: {error}, waiting 5 secs");
        sleep(Duration::from_secs(5)).await;
    }
}

async fn up_s3_client(config: &Storage) -> aws_sdk_s3::Client {
    let creds = Credentials::new(
        config.access_key.expose_secret(),
        config.secret_key.expose_secret(),
        None,
        None,
        "config",
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
    let redis = up_redis(&CONFIG.redis).await?;

    let token_service = JwtTokenService::new(CONFIG.jwt.expires, CONFIG.jwt.secret.expose_secret());

    let files_storage = S3FilesStorage::new(
        up_s3_client(&CONFIG.storage).await,
        CONFIG.storage.bucket.clone().into(),
    );

    let files_service = BasicFilesService::new(
        files_storage,
        Cache::new(
            redis.clone(),
            db.clone(),
            CONFIG.caches.files.as_secs() as _,
        ),
        RandomIdGeneratorService,
        CONFIG.limits.max_filesize.as_u64(),
    );

    let folders_service = BasicFoldersService::new(
        Cache::new(
            redis.clone(),
            db.clone(),
            CONFIG.caches.folders.as_secs() as _,
        ),
        files_service.clone(),
        RandomIdGeneratorService,
    );

    let download_service =
        BasicDownloadService::new(files_service.clone(), folders_service.clone());

    Server::builder()
        .add_service(FolderServiceServer::new(BasicGrpcFolderService::new(
            files_service.clone(),
            folders_service.clone(),
            token_service,
        )))
        .add_service(FilesServiceServer::new(BasicGrpcFilesService::new(
            files_service,
            folders_service,
            download_service,
        )))
        .serve("[::1]:50051".parse()?)
        .await?;

    Ok(())
}
