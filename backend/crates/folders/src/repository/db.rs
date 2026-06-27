use crate::repository::FoldersRepository;
use domain::entity::folders;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

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

    async fn insert(&self, folder: folders::ActiveModel) -> Result<folders::Model, Self::Error> {
        folder.insert(self).await
    }

    async fn update(&self, folder: folders::ActiveModel) -> Result<folders::Model, Self::Error> {
        folder.update(self).await
    }

    async fn delete(&self, folder_id: folders::Id) -> Result<Option<folders::Model>, Self::Error> {
        folders::Entity::delete_by_id(folder_id)
            .exec_with_returning(self)
            .await
    }

    async fn rename(
        &self,
        folder_id: folders::Id,
        encrypted_name: String,
    ) -> Result<Option<folders::Model>, Self::Error> {
        let model = folders::ActiveModel {
            id: Set(folder_id),
            encrypted_name: Set(encrypted_name),
            ..Default::default()
        };

        match folders::Entity::update(model).validate()?.exec(self).await {
            Ok(x) => Ok(Some(x)),
            Err(sea_orm::DbErr::RecordNotFound(_)) => Ok(None),
            Err(err) => Err(err),
        }
    }
}
