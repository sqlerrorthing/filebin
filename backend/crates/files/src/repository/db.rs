use sea_orm::{ActiveModelTrait, DatabaseConnection};
use domain::entity::files;
use crate::repository::FilesRepository;

impl FilesRepository for DatabaseConnection {
    type Error = sea_orm::DbErr;

    async fn insert(&self, folder: files::ActiveModel) -> Result<files::Model, Self::Error> {
        folder.insert(self).await
    }

    async fn update(&self, folder: files::ActiveModel) -> Result<files::Model, Self::Error> {
        folder.update(self).await
    }
}
