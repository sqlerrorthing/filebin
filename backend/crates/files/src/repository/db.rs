use crate::repository::FilesRepository;
use domain::persistance::{files, folders};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, SelectExt};

impl FilesRepository for DatabaseConnection {
    type Error = sea_orm::DbErr;

    async fn files_count(&self, folder_id: folders::Id) -> Result<u64, Self::Error> {
        files::Entity::find()
            .filter(files::Column::FolderId.eq(folder_id))
            .count(self)
            .await
    }

    async fn delete_files_from_folder(
        &self,
        folder_id: folders::Id,
    ) -> Result<Vec<files::Model>, Self::Error> {
        files::Entity::delete_many()
            .filter(files::Column::FolderId.eq(folder_id))
            .exec_with_returning(self)
            .await
    }

    async fn delete_file(&self, file_id: files::Id) -> Result<Option<files::Model>, Self::Error> {
        files::Entity::delete_by_id(file_id)
            .exec_with_returning(self)
            .await
    }

    async fn find_file_by_public_id(
        &self,
        public_id: files::PublicId,
    ) -> Result<Option<files::Model>, Self::Error> {
        files::Entity::find()
            .filter(files::Column::PublicId.eq(public_id))
            .one(self)
            .await
    }

    async fn list_folder_files(
        &self,
        folder_id: folders::Id,
    ) -> Result<Vec<files::Model>, Self::Error> {
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
