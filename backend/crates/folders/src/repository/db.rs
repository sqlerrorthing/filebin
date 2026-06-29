use crate::repository::FoldersRepository;
use domain::{models, persistence};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

struct CastedModel(models::folders::Model);

impl From<(persistence::folders::Model, models::encrypted_blobs::Model)> for CastedModel {
    fn from(
        (model, encrypted_name): (persistence::folders::Model, models::encrypted_blobs::Model),
    ) -> Self {
        CastedModel(models::folders::Model {
            id: model.id,
            public_id: model.public_id,
            encrypted_name,
            expired_at: model.expired_at.map(|t| t.to_utc()),
            created_at: model.created_at.to_utc(),
        })
    }
}

impl FoldersRepository for DatabaseConnection {
    type Error = sea_orm::DbErr;

    async fn find_folder_by_public_id(
        &self,
        public_id: models::folders::PublicId,
    ) -> Result<Option<models::folders::Model>, Self::Error> {
        let res = persistence::folders::Entity::load()
            .filter(persistence::folders::Column::PublicId.eq(public_id))
            .with
            .one(self)
            .await?;

        todo!()
    }

    async fn new_folder(
        &self,
        folder: models::folders::NewFolder,
    ) -> Result<models::folders::Model, Self::Error> {
        folder.insert(self).await
    }

    async fn delete(&self, folder_id: folders::Id) -> Result<Option<folders::Model>, Self::Error> {
        persistence::folders::Entity::delete_by_id(folder_id)
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
