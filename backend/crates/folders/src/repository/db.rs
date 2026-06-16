use derive_new::new;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use domain::entity::folders;
use domain::entity::folders::ActiveModel;
use crate::repository::FoldersRepository;

#[derive(Debug, Clone, new)]
pub struct DbFoldersRepository(DatabaseConnection);

impl FoldersRepository for DbFoldersRepository {
    type Error = sea_orm::DbErr;

    async fn insert(&self, folder: ActiveModel) -> Result<folders::Model, Self::Error> {
        folder.insert(&self.0).await
    }

    async fn update(&self, folder: ActiveModel) -> Result<folders::Model, Self::Error> {
        folder.update(&self.0).await
    }
}
