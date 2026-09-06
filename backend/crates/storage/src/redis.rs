use crate::{SetTtl, Storage};
use deadpool_redis::redis::{AsyncCommands, RedisResult, SetExpiry, SetOptions};
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

    async fn set<K, V>(&self, key: K, value: &V, ttl: impl Into<SetTtl> + Send) -> Result<(), Self::Error>
    where
        K: Into<String>,
        V: Serialize,
    {
        let mut conn = self.get().await?;
        let serialized = postcard::to_allocvec(value)?;
        let key = key.into();

        let expiration = match ttl.into() {
            SetTtl::Keep => Some(SetExpiry::KEEPTTL),
            SetTtl::Set(seconds) => seconds.map(|seconds| SetExpiry::EX(seconds as u64))
        };

        let mut options = SetOptions::default();
        if let Some(expiration) = expiration {
            options = options.with_expiration(expiration)
        }

        let _: () = conn
            .set_options(key, serialized, options).await.map_err(PoolError::Backend)?;

        Ok(())
    }

    async fn set_ex<K>(&self, key: K, ttl: Option<u32>) -> Result<bool, Self::Error>
    where
        K: Into<String> + Send
    {
        let mut conn = self.get().await?;
        let key = key.into();

        let result: RedisResult<i32> = match ttl {
            Some(seconds) => {
                conn.expire(&key, seconds as i64).await
            }
            None => {
                conn.persist(&key).await
            }
        };

        Ok(matches!(result, Ok(1)))
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

    async fn bulk_delete<I>(&self, keys: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item: Into<String> + Send>,
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
