use crate::repository::FilesRepository;
use domain::entity::{files, folders};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

impl FilesRepository for DatabaseConnection {
    type Error = sea_orm::DbErr;

    async fn delete_files_from_folder(
        &self,
        folder_id: folders::Id,
    ) -> Result<Vec<files::Model>, Self::Error> {
        files::Entity::delete_many()
            .filter(files::Column::FolderId.eq(folder_id))
            .exec_with_returning(self)
            .await
    }

    async fn list_folder_files(&self, folder_id: folders::Id) -> Result<Vec<files::Model>, Self::Error> {
        files::Entity::find()
            .filter(files::Column::FolderId.eq(folder_id))
            .all(self)
            .await
    }

    async fn insert(&self, folder: files::ActiveModel) -> Result<files::Model, Self::Error> {
        folder.insert(self).await
    }

    async fn update(&self, folder: files::ActiveModel) -> Result<files::Model, Self::Error> {
        folder.update(self).await
    }
}
