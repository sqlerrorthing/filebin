use crate::repository::FoldersRepository;
use domain::{models, persistence};
use sea_orm::ColumnTrait;
use sea_orm::{DatabaseConnection, EntityTrait};
use sea_orm::{IntoActiveModel, QueryFilter};
use std::any::{type_name, type_name_of_val};
use std::sync::Arc;

impl FoldersRepository for DatabaseConnection {
    type Error = sea_orm::DbErr;

    async fn find_folder_by_public_id(
        &self,
        public_id: models::folders::PublicId,
    ) -> Result<Option<models::folders::Model>, Self::Error> {
        let Some(folder) = persistence::folders::Entity::load()
            .filter(persistence::folders::Column::PublicId.eq(public_id))
            .with((
                persistence::encrypted_blobs::Entity,
                persistence::encrypted_vault::Entity,
            ))
            .one(self)
            .await?
        else {
            return Ok(None);
        };

        let from = type_name_of_val(&folder);
        folder.into_active_model().try_into().map_err(|e| {
            sea_orm::DbErr::TryIntoErr {
                from,
                into: type_name::<models::folders::Model>(),
                source: Arc::new(e),
            }
        }).map(Some)
    }

    async fn new_folder(
        &self,
        new_folder: models::folders::NewFolder,
    ) -> Result<models::folders::Model, Self::Error> {
        let folder = persistence::folders::ActiveModel::builder()
            .set_public_id(new_folder.public_id)
            .set_expired_at(new_folder.expired_at.map(|f| f.fixed_offset()))
            .set_encrypted_blobs(new_folder.encrypted_name)
            .save(self)
            .await?;

        let from = type_name_of_val(&folder);
        folder.try_into().map_err(|e| sea_orm::DbErr::TryIntoErr {
            from,
            into: type_name::<models::folders::Model>(),
            source: Arc::new(e),
        })
    }

    async fn delete(
        &self,
        folder_id: models::folders::Id,
    ) -> Result<Option<models::folders::Model>, Self::Error> {
        // let Some(res) = persistence::folders::Entity::delete_by_id(folder_id)
        //     .exec_with_returning(self)
        //     .await?;

        todo!()
    }

    async fn rename(
        &self,
        folder_id: models::folders::Id,
        new_name: models::encrypted_blobs::NewBlob,
    ) -> Result<Option<models::folders::Model>, Self::Error> {
        // let model = folders::ActiveModel {
        //     id: Set(folder_id),
        //     encrypted_name: Set(encrypted_name),
        //     ..Default::default()
        // };
        //
        // match folders::Entity::update(model).validate()?.exec(self).await {
        //     Ok(x) => Ok(Some(x)),
        //     Err(sea_orm::DbErr::RecordNotFound(_)) => Ok(None),
        //     Err(err) => Err(err),
        // }
        todo!()
    }
}
