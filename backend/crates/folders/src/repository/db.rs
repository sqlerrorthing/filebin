use std::default::Default;
use crate::repository::FoldersRepository;
use domain::{models, persistence};
use sea_orm::{ActiveModelTrait, ColumnTrait, Set, TryIntoModel};
use sea_orm::{DatabaseConnection, EntityTrait};
use sea_orm::QueryFilter;

impl FoldersRepository for DatabaseConnection {
    type Error = sea_orm::DbErr;

    async fn find_folder_by_public_id(
        &self,
        public_id: models::folders::PublicId,
    ) -> Result<Option<models::folders::Model>, Self::Error> {
        let Some(folder) = persistence::folders::Entity::find()
            .filter(persistence::folders::Column::PublicId.eq(public_id))
            .one(self)
            .await?
        else {
            return Ok(None);
        };

        Ok(Some(folder.into()))
    }

    async fn new_folder(
        &self,
        new_folder: models::folders::NewFolder,
    ) -> Result<models::folders::Model, Self::Error> {
        let folder= persistence::folders::ActiveModel {
            public_id: Set(new_folder.public_id),
            encrypted_name: Set(new_folder.encrypted_name),
            expired_at: Set(new_folder.expired_at.map(|d| d.fixed_offset())),
            ..Default::default()
        }.save(self).await?;

        Ok(folder.try_into_model()?.into())
    }

    async fn delete(
        &self,
        folder_id: models::folders::Id,
    ) -> Result<Option<models::folders::Model>, Self::Error> {
        let res = persistence::folders::Entity::delete_by_id(folder_id)
            .exec_with_returning(self)
            .await?;

        Ok(res.map(Into::into))
    }

    async fn rename(
        &self,
        folder_id: models::folders::Id,
        new_name: models::folders::FolderName,
    ) -> Result<Option<models::folders::Model>, Self::Error> {
        let model = persistence::folders::ActiveModel {
            id: Set(folder_id),
            encrypted_name: Set(new_name),
            ..Default::default()
        };

        match persistence::folders::Entity::update(model).validate()?.exec(self).await {
            Ok(x) => Ok(Some(x.into())),
            Err(sea_orm::DbErr::RecordNotFound(_)) => Ok(None),
            Err(err) => Err(err),
        }
    }
}
