use crate::repository::FoldersRepository;
use domain::entity::folders;
use domain::entity::folders::ActiveModel;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

impl FoldersRepository for DatabaseConnection {
    type Error = sea_orm::DbErr;

    async fn find_folder_by_public_id(
        &self,
        public_id: folders::PublicId,
    ) -> Result<Option<folders::Model>, Self::Error> {
        folders::Entity::find()
            .filter(folders::Column::PublicId.eq(public_id))
            .one(self)
            .await
    }

    async fn insert(&self, folder: ActiveModel) -> Result<folders::Model, Self::Error> {
        folder.insert(self).await
    }

    async fn update(&self, folder: ActiveModel) -> Result<folders::Model, Self::Error> {
        folder.update(self).await
    }

    async fn delete(&self, folder_id: folders::Id) -> Result<Option<folders::Model>, Self::Error> {
        folders::Entity::delete_by_id(folder_id)
            .exec_with_returning(self)
            .await
    }
}
