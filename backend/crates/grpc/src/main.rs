#![feature(min_specialization, impl_trait_in_assoc_type)]

use std::any::type_name_of_val;
use amqprs::connection::Connection;
use crate::config::{CONFIG, Db, Redis, Storage};
use crate::schema::api::folder::v1::files_service_server::FilesServiceServer;
use crate::schema::api::folder::v1::folder_service_server::FolderServiceServer;
use crate::sealed::Leaked;
use crate::v1::files::BasicGrpcFilesService;
use crate::v1::folder::BasicGrpcFolderService;
use auth::service::jwt::JwtTokenService;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use cache::Cache;
use deadpool_redis::Runtime;
use download::service::basic::BasicDownloadService;
use files::service::basic::BasicFilesService;
use files::storage::s3::S3FilesStorage;
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
use updates::service::basic::LocalUpdatesService;
use updates::service::DynUpdatesService;
use updates::service::rabbitmq::RabbitMQUpdatesService;
use upload::service::basic::{BasicUploadService, Limits, LimitsBuilder};

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

async fn up_redis(config: &Redis) -> color_eyre::Result<deadpool_redis::Pool> {
    let cfg = deadpool_redis::Config::from_url(&config.url);
    let builder = cfg.builder()?.runtime(Runtime::Tokio1);

    Ok(builder.build()?)
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

async fn up_rabbitmq() -> color_eyre::Result<Option<Connection>> {
    let Some(url) = &CONFIG.rabbitmq.url else {
        return Ok(None)
    };

    Ok(Some(Connection::open(&url.as_str().try_into()?).await?))
}

fn tracing() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init()
}

mod sealed {
    /// Leaking self due to this will alive all runtime time
    pub trait Leaked {
        fn leaked(self) -> &'static Self;
    }

    impl<T> Leaked for T {
        fn leaked(self) -> &'static Self {
            Box::leak(Box::new(self))
        }
    }
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing();

    let db = up_db(&CONFIG.db).await?.leaked();
    let redis = up_redis(&CONFIG.redis).await?.leaked();
    info!("Redis connected");
    let rabbitmq = up_rabbitmq().await?
        .inspect(|_| info!("RabbitMQ connected"));

    let updates_service: &dyn DynUpdatesService = if let Some(conn) = rabbitmq {
        RabbitMQUpdatesService::new(conn).leaked()
    } else {
        LocalUpdatesService::new(100).leaked()
    };
    
    let token_service =
        JwtTokenService::new(CONFIG.jwt.expires, CONFIG.jwt.secret.expose_secret()).leaked();

    let files_storage = S3FilesStorage::new(
        up_s3_client(&CONFIG.storage).await,
        CONFIG.storage.bucket.clone().into(),
    )
    .leaked();

    let files_service = BasicFilesService::new(
        files_storage,
        Cache::new(redis, db, CONFIG.caches.files.as_secs() as _),
        RandomIdGeneratorService,
    )
    .leaked();

    let folders_service = BasicFoldersService::new(
        Cache::new(redis, db, CONFIG.caches.folders.as_secs() as _),
        files_service,
        RandomIdGeneratorService,
        updates_service,
    )
    .leaked();

    let download_service = BasicDownloadService::new(files_service, folders_service);

    let upload_service = BasicUploadService::new(
        files_service,
        folders_service,
        token_service,
        updates_service,
        LimitsBuilder::default()
            .max_filesize(CONFIG.limits.max_filesize.as_u64())
            .max_files_per_folder(CONFIG.limits.max_files_per_folder)
            .build()?,
    );

    dbg!(type_name_of_val(&upload_service));

    Server::builder()
        .add_service(FolderServiceServer::new(BasicGrpcFolderService::new(
            folders_service,
            token_service,
            updates_service
        )))
        .add_service(FilesServiceServer::new(BasicGrpcFilesService::new(
            files_service,
            folders_service,
            download_service,
            upload_service,
        )))
        .serve("0.0.0.0:50051".parse()?)
        .await?;

    Ok(())
}
