use crate::repository::FilesRepository;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, JoinType, PaginatorTrait, QueryFilter, QuerySelect, RelationTrait};
use domain::{models, persistence};

impl FilesRepository for DatabaseConnection {
    type Error = sea_orm::DbErr;

    async fn files_count(&self, folder_id: models::folders::Id) -> Result<u64, Self::Error> {
        persistence::files::Entity::find()
            .filter(persistence::files::Column::FolderId.eq(folder_id))
            .count(self)
            .await
    }

    async fn delete_files_from_folder(
        &self,
        folder_id: models::folders::Id,
    ) -> Result<Vec<models::files::Model>, Self::Error> {
        todo!()
    }

    async fn delete_file(&self, file_id: models::files::Id) -> Result<Option<models::files::Model>, Self::Error> {
        todo!()

    }

    async fn find_file_by_public_id(
        &self,
        public_id: models::files::PublicId,
    ) -> Result<Option<models::files::Model>, Self::Error> {
        todo!()

    }

    async fn list_folder_files(
        &self,
        folder_id: models::folders::Id,
    ) -> Result<Vec<models::files::Model>, Self::Error> {
        todo!()

    }

    async fn new_file(&self, _new_file: models::files::NewFile) -> Result<models::files::Model, Self::Error> {
        todo!()
    }
}
