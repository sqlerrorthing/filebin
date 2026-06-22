use crate::Storage;
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::{Pool, PoolError};
use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("serializer error: {0}")]
    Serialize(#[from] postcard::Error),
    #[error("pool error: {0}")]
    Pool(#[from] PoolError),
}

impl Storage for Pool {
    type Error = Error;

    async fn set<K, V>(&self, key: K, value: &V, ttl: Option<u32>) -> Result<(), Self::Error>
    where
        K: Into<String>,
        V: Serialize,
    {
        let mut conn = self.get().await?;
        let serialized = postcard::to_allocvec(value)?;
        let key = key.into();

        match ttl {
            Some(seconds) => {
                let _: () = conn
                    .set_ex(key, serialized, seconds as u64)
                    .await
                    .map_err(PoolError::Backend)?;
            }
            None => {
                let _: () = conn
                    .set(key, serialized)
                    .await
                    .map_err(PoolError::Backend)?;
            }
        }

        Ok(())
    }

    async fn get<K, V>(&self, key: K) -> Result<Option<V>, Self::Error>
    where
        K: Into<String>,
        V: DeserializeOwned,
    {
        let mut conn = self.get().await?;
        let value: Option<Vec<u8>> = conn.get(key.into()).await.map_err(PoolError::Backend)?;
        Ok(value.map(|m| postcard::from_bytes(&m)).transpose()?)
    }

    async fn delete<K>(&self, key: K) -> Result<bool, Self::Error>
    where
        K: Into<String>,
    {
        let mut conn = self.get().await?;
        let res: usize = conn.del(key.into()).await.map_err(PoolError::Backend)?;
        Ok(res == 1)
    }

    async fn bulk_delete<K, I>(&self, keys: I) -> Result<(), Self::Error>
    where
        K: Into<String> + Send,
        I: IntoIterator<Item = K>,
    {
        let keys = keys.into_iter().map(Into::into).collect::<Vec<_>>();
        if keys.is_empty() {
            return Ok(())
        }
        
        let mut conn = self.get().await?;
        let _: usize = conn
            .unlink(keys)
            .await
            .map_err(PoolError::Backend)?;
        Ok(())
    }
}
