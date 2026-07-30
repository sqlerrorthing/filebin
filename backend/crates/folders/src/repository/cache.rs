use crate::repository::FoldersRepository;
use cache::{Cache, Cached};
use storage::Storage;
use domain::models::{encrypted_blobs, folders};

const PREFIX: &str = "cache:folders";

fn key_by_id(folder_id: folders::Id) -> String {
    format!("{PREFIX}:f:{folder_id}")
}

fn key_by_public_id(public_id: &folders::PublicId) -> String {
    format!("{PREFIX}:pub:{public_id}")
}

impl<S, R> FoldersRepository for Cache<S, R>
where
    S: Storage,
    R: FoldersRepository,
{
    type Error = R::Error;

    async fn find_folder_by_public_id(
        &self,
        public_id: folders::PublicId,
    ) -> Result<Option<folders::Model>, Self::Error> {
        self.load_or_cache(key_by_public_id(&public_id), async |repo| {
            repo.find_folder_by_public_id(public_id).await.map(Cached)
        })
        .await
        .map(|v| v.0)
    }

    async fn new_folder(&self, new_folder: folders::NewFolder) -> Result<folders::Model, Self::Error> {
        self.repository().new_folder(new_folder).await
    }

    async fn delete(&self, folder_id: folders::Id) -> Result<Option<folders::Model>, Self::Error> {
        let folder = self.repository().delete(folder_id).await?;
        if let Some(folder) = &folder {
            self.clear_cache_keys([key_by_id(folder.id), key_by_public_id(&folder.public_id)])
                .await;
        }
        Ok(folder)
    }

    async fn rename(&self, folder_id: folders::Id, new_name: encrypted_blobs::NewBlob) -> Result<Option<folders::Model>, Self::Error> {
        let folder = self.repository().rename(folder_id, new_name).await?;
        if let Some(folder) = &folder {
            self.clear_cache_keys([key_by_id(folder.id), key_by_public_id(&folder.public_id)])
                .await;
        }
        Ok(folder)
    }
}
