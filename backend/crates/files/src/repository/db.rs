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
        let query = persistence::files::Entity::find()
            .filter(persistence::files::Column::FolderId.eq(folder_id));

        let query = query.join(
            JoinType::LeftJoin,
            persistence::files::Relation::EncryptedVault.def()
        );
        let query = query.join(
            JoinType::LeftJoin,
            persistence::files::Relation::EncryptedBlobs.def()
        );
        let query = query.join(
            JoinType::LeftJoin,
            persistence::encrypted_blobs::Relation::EncryptedVault.def()
        );

        let result = query
            .select_also(persistence::encrypted_vault::Entity) // Соответствует первому join (data_meta)
            .select_also(persistence::encrypted_blobs::Entity) // Соответствует второму join (blob)
            .select_also(persistence::encrypted_vault::Entity) // Соответствует третьему join (blob_meta)
            .all(self)
            .await?;
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
