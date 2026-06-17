use crate::repository::FoldersRepository;
use crate::service::FoldersService;
use derive_new::new;
use domain::entity::folders;
use id_generator::service::IdGeneratorService;
use sea_orm::sea_query::prelude::Utc;
use sea_orm::{NotSet, Set};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, new)]
pub struct BasicFoldersService<FR, IGS> {
    folder_repository: FR,
    id_generator_service: IGS,
}

#[derive(Debug, Error)]
pub enum Error<FR: FoldersRepository> {
    #[error("folders repository error: {0}")]
    Repository(#[source] FR::Error),
}

impl<FR: FoldersRepository, IGS: IdGeneratorService> FoldersService
    for BasicFoldersService<FR, IGS>
{
    type Error = Error<FR>;

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
            todo!("delete entire folder");
            return Ok(None)
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
