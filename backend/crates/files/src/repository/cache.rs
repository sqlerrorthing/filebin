use crate::repository::FilesRepository;
use cache::Cache;
use domain::entity::{files, folders};
use storage::Storage;

const PREFIX: &str = "cache:files";

fn key_by_id(files_id: files::Id) -> String {
    format!("{PREFIX}:file:{files_id}")
}

fn key_by_public_id(files_id: &files::PublicId) -> String {
    format!("{PREFIX}:file:pub:{files_id}")
}

fn folder_files_key(folder_id: folders::Id) -> String {
    format!("{PREFIX}:folder:{folder_id}")
}

fn folder_files_count_key(folder_id: folders::Id) -> String {
    format!("{PREFIX}:folder:count:{folder_id}")
}

impl<S, R> FilesRepository for Cache<S, R>
where
    S: Storage,
    R: FilesRepository,
{
    type Error = R::Error;

    async fn files_count(&self, folder_id: folders::Id) -> Result<u64, Self::Error> {
        self.load_or_cache(folder_files_count_key(folder_id), async |repo| {
            repo.files_count(folder_id).await
        })
        .await
    }

    async fn delete_files_from_folder(
        &self,
        folder_id: folders::Id,
    ) -> Result<Vec<files::Model>, Self::Error> {
        let result = self
            .repository()
            .delete_files_from_folder(folder_id)
            .await?;

        self.clear_cache_keys(result.iter().flat_map(|f| {
            [
                key_by_id(f.id),
                folder_files_key(folder_id),
                key_by_public_id(&f.public_id),
                folder_files_count_key(folder_id),
            ]
        }))
        .await;

        Ok(result)
    }

    async fn delete_file(&self, file_id: files::Id) -> Result<Option<files::Model>, Self::Error> {
        let file = self.repository().delete_file(file_id).await?;
        if let Some(file) = &file {
            self.clear_cache_keys([
                key_by_id(file.id),
                folder_files_key(file.folder_id),
                key_by_public_id(&file.public_id),
                folder_files_count_key(file.folder_id)
            ]).await;
        }

        Ok(file)
    }

    async fn find_file_by_public_id(
        &self,
        public_id: files::PublicId,
    ) -> Result<Option<files::Model>, Self::Error> {
        self.load_or_cache(key_by_public_id(&public_id), async |repo| {
            repo.find_file_by_public_id(public_id).await
        })
        .await
    }

    async fn list_folder_files(
        &self,
        folder_id: folders::Id,
    ) -> Result<Vec<files::Model>, Self::Error> {
        self.load_or_cache(folder_files_key(folder_id), async |repo| {
            repo.list_folder_files(folder_id).await
        })
        .await
    }

    async fn insert(&self, file: files::ActiveModel) -> Result<files::Model, Self::Error> {
        let file = self.repository().insert(file).await?;
        self.clear_cache_keys([
            key_by_id(file.id),
            folder_files_key(file.folder_id),
            key_by_public_id(&file.public_id),
            folder_files_count_key(file.folder_id)
        ])
        .await;
        Ok(file)
    }

    async fn update(&self, file: files::ActiveModel) -> Result<files::Model, Self::Error> {
        let file = self.repository().update(file).await?;
        self.clear_cache_keys([
            key_by_id(file.id),
            folder_files_key(file.folder_id),
            key_by_public_id(&file.public_id),
            folder_files_count_key(file.folder_id)
        ]).await;
        Ok(file)
    }
}
