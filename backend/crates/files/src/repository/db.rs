use derive_new::new;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use domain::entity::files;
use crate::repository::FilesRepository;

#[derive(Debug, Clone, new)]
pub struct DbFilesRepository(DatabaseConnection);

impl FilesRepository for DbFilesRepository {
    type Error = sea_orm::DbErr;

    async fn insert(&self, folder: files::ActiveModel) -> Result<files::Model, Self::Error> {
        folder.insert(&self.0).await
    }

    async fn update(&self, folder: files::ActiveModel) -> Result<files::Model, Self::Error> {
        folder.update(&self.0).await
    }
}
