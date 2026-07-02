use crate::repository::FoldersRepository;
use domain::{models, persistence};
use sea_orm::{DatabaseConnection, EntityTrait};

impl FoldersRepository for DatabaseConnection {
    type Error = sea_orm::DbErr;

    async fn find_folder_by_public_id(
        &self,
        public_id: models::folders::PublicId,
    ) -> Result<Option<models::folders::Model>, Self::Error> {
    //     let Some(res) = persistence::folders::Entity::load()
    //         .filter(persistence::folders::Column::PublicId.eq(public_id))
    //         .with(persistence::encrypted_blobs::Entity)
    //         .one(self)
    //         .await?
    //     else {
    //         Ok(None)
    //     };

        // dbg!(res);
        todo!()
    }

    async fn new_folder(
        &self,
        folder: models::folders::NewFolder,
    ) -> Result<models::folders::Model, Self::Error> {
        let folder = persistence::folders::ActiveModel::builder()
            .set_public_id(folder.public_id)
            .set_expired_at(folder.expired_at.map(|f| f.fixed_offset()))
            .set_encrypted_blobs(folder.encrypted_name)
            .save(self)
            .await?;

        todo!("{:?}", dbg!(folder))
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
