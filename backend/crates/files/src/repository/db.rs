use crate::repository::FilesRepository;
use byte_unit::Byte;
use domain::{models, persistence};
use futures_util::FutureExt;
use sea_orm::ExprTrait;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QuerySelect, Set,
};

impl FilesRepository for DatabaseConnection {
    type Error = sea_orm::DbErr;

    async fn files_count(&self, folder_id: models::folders::Id) -> Result<u64, Self::Error> {
        persistence::files::Entity::find()
            .filter(persistence::files::Column::FolderId.eq(folder_id))
            .count(self)
            .await
    }

    async fn files_size(
        &self,
        folder_id: models::folders::Id,
    ) -> Result<Option<Byte>, Self::Error> {
        Ok(persistence::files::Entity::find()
            .filter(persistence::files::Column::FolderId.eq(folder_id))
            .select_only()
            .expr_as(
                Expr::col(persistence::files::Column::FileSize).sum(),
                "files_size",
            )
            .into_tuple::<Option<i64>>()
            .one(self)
            .await?
            .flatten()
            .and_then(Byte::from_i64))
    }

    async fn delete_files_from_folder(
        &self,
        folder_id: models::folders::Id,
    ) -> Result<Vec<models::files::Model>, Self::Error> {
        persistence::files::Entity::delete_many()
            .filter(persistence::files::Column::FolderId.eq(folder_id))
            .exec_with_returning(self)
            .await
            .map(|res| res.into_iter().map(Into::into).collect())
    }

    async fn delete_file(
        &self,
        file_id: models::files::Id,
    ) -> Result<Option<models::files::Model>, Self::Error> {
        persistence::files::Entity::delete_by_id(file_id)
            .exec_with_returning(self)
            .await
            .map(|opt| opt.map(Into::into))
    }

    async fn find_file_by_public_id(
        &self,
        public_id: models::files::PublicId,
    ) -> Result<Option<models::files::Model>, Self::Error> {
        persistence::files::Entity::find()
            .filter(persistence::files::Column::PublicId.eq(public_id))
            .one(self)
            .await
            .map(|opt| opt.map(Into::into))
    }

    async fn list_folder_files(
        &self,
        folder_id: models::folders::Id,
    ) -> Result<Vec<models::files::Model>, Self::Error> {
        persistence::files::Entity::find()
            .filter(persistence::files::Column::FolderId.eq(folder_id))
            .all(self)
            .await
            .map(|res| res.into_iter().map(Into::into).collect())
    }

    async fn new_file(
        &self,
        new_file: models::files::NewFile,
    ) -> Result<models::files::Model, Self::Error> {
        persistence::files::Entity::insert(persistence::files::ActiveModel {
            public_id: Set(new_file.public_id),
            folder_id: Set(new_file.folder_id),
            data_meta: Set(new_file.data_meta),
            meta: Set(new_file.meta),
            storage_path: Set(new_file.storage_path),
            file_size: Set(new_file.file_size),
            ..Default::default()
        })
        .exec_with_returning(self)
        .await
        .map(Into::into)
    }
}
