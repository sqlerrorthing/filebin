use crate::repository::FoldersRepository;
use crate::service::FoldersService;
use derive_new::new;
use domain::entity::folders;
use files::service::FilesService;
use id_generator::service::IdGeneratorService;
use sea_orm::sea_query::prelude::Utc;
use sea_orm::{NotSet, Set};
use std::hint::cold_path;
use std::time::Duration;
use thiserror::Error;
use tokio::spawn;
use tracing::{error, info};

#[derive(Debug, Clone, new)]
pub struct BasicFoldersService<FR, FS, IGS> {
    folder_repository: FR,
    files_service: FS,
    id_generator_service: IGS,
}

#[derive(Debug, Error)]
pub enum Error<FR: FoldersRepository, FS: FilesService> {
    #[error("folders repository error: {0}")]
    Repository(#[source] FR::Error),
    #[error("files service error: {0}")]
    Files(#[source] FS::Error),
}

impl<FR, FS, IGS> FoldersService for BasicFoldersService<FR, FS, IGS>
where
    FR: FoldersRepository,
    FS: FilesService,
    IGS: IdGeneratorService,
    Self: Clone,
{
    type Error = Error<FR, FS>;

    async fn remove_entire_folder(&self, folder_id: folders::Id) -> Result<bool, Self::Error> {
        self.files_service
            .delete_files_from_folder(folder_id)
            .await
            .map_err(Error::Files)?;

        self.folder_repository
            .delete(folder_id)
            .await
            .map_err(Error::Repository)
            .map(|f| f.is_some())
    }

    async fn find_folder_by_public_id(
        &self,
        public_id: folders::PublicId,
    ) -> Result<Option<folders::Model>, Self::Error> {
        let folder = self
            .folder_repository
            .find_folder_by_public_id(public_id)
            .await
            .map_err(Error::Repository)?;

        if let Some(folder) = &folder
            && folder.expired_at.is_some_and(|exp| Utc::now() > exp)
        {
            cold_path();
            info!("Expired folder found, deleting");
            let folder_id = folder.id;
            let this = self.clone();
            spawn(async move {
                if let Err(err) = this.remove_entire_folder(folder_id).await {
                    error!(folder = %folder_id, error = %err, "Failed to remove expired folder");
                }
            });
            return Ok(None);
        }

        Ok(folder)
    }

    async fn create_folder(
        &self,
        encrypted_name: String,
        expires: Option<Duration>,
    ) -> Result<folders::Model, Self::Error> {
        let model = folders::ActiveModel {
            public_id: Set(self.id_generator_service.next_public_folder_id()),
            encrypted_name: Set(encrypted_name),
            expired_at: expires.map_or(NotSet, |expires| Set(Some((Utc::now() + expires).into()))),
            created_at: Set(Utc::now().into()),
            ..Default::default()
        };

        self.folder_repository
            .insert(model)
            .await
            .map_err(Error::Repository)
    }
}
