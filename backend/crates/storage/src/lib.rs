pub mod redis;

use serde::Serialize;
use serde::de::DeserializeOwned;
use service::service;

pub enum SetTtl {
    Keep,
    Set(Option<u32>)
}

impl From<Option<u32>> for SetTtl {
    fn from(value: Option<u32>) -> Self {
        Self::Set(value)
    }
}

impl From<u32> for SetTtl {
    fn from(value: u32) -> Self {
        Self::Set(Some(value))
    }
}

#[service]
pub trait Storage: Clone {
    type Error;

    #[result]
    async fn set<K, V>(&self, key: K, value: &V, ttl: impl Into<SetTtl> + Send)
    where
        K: Into<String> + Send,
        V: Serialize + Sync;

    #[result]
    async fn set_ex<K>(&self, key: K, ttl: Option<u32>) -> bool
    where
        K: Into<String> + Send;

    #[result]
    async fn get<K, V>(&self, key: K) -> Option<V>
    where
        K: Into<String> + Send,
        V: DeserializeOwned;

    #[result]
    async fn delete<K>(&self, key: K) -> bool
    where
        K: Into<String> + Send;

    #[result]
    async fn bulk_delete<I>(&self, keys: I)
    where
        I: IntoIterator<Item: Into<String> + Send, IntoIter: Send> + Send;
}
