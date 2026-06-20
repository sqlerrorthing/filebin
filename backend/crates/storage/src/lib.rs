pub mod redis;

use serde::de::DeserializeOwned;
use serde::Serialize;
use service::service;

#[service]
pub trait Storage: 'static {
    type Error;

    #[result]
    async fn set<K, V>(
        &self,
        key: K,
        value: &V,
        ttl: Option<u32>
    )
    where
        K: Into<String> + Send,
        V: Serialize + Sync;

    #[result]
    async fn get<K, V>(
        &self,
        key: K
    ) -> Option<V>
    where
        K: Into<String> + Send,
        V: DeserializeOwned;

    #[result]
    async fn delete<K>(&self, key: K) -> bool
    where
        K: Into<String> + Send;

    #[result]
    async fn bulk_delete<K, I>(&self, keys: I)
    where
        K: Into<String> + Send,
        I: IntoIterator<Item = K> + Send,
        I::IntoIter: ExactSizeIterator + Send;
}
