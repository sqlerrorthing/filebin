use crate::config::{CONFIG, Db};
use crate::schema::api::folder::v1::folder_service_server::FolderServiceServer;
use crate::v1::folder::BasicGrpcFolderService;
use auth::service::jwt::JwtTokenService;
use folders::repository::db::DbFoldersRepository;
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

    let folders_repository = DbFoldersRepository::new(db);

    let token_service = JwtTokenService::new(
        chrono::Duration::from_std(CONFIG.jwt.expires)?,
        &CONFIG.jwt.secret.expose_secret(),
    );

    let folders_service = BasicFoldersService::new(folders_repository, RandomIdGeneratorService);

    Server::builder()
        .add_service(FolderServiceServer::new(BasicGrpcFolderService::new(
            folders_service,
            token_service,
        )))
        .serve("[::1]:50051".parse()?)
        .await?;

    Ok(())
}
