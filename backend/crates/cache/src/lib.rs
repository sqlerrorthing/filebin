use std::any::type_name;
use derive_builder::Builder;
use derive_new::new;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::Arc;
use storage::Storage;
use tokio::sync::Notify;
use tracing::{error, trace};

type InflightMap = parking_lot::Mutex<HashMap<String, Arc<Notify>>>;

#[derive(Serialize, Deserialize)]
pub struct Cached<T>(pub Option<T>);

#[derive(Debug, Clone, Builder, new)]
pub struct Cache<S: Storage, R> {
    storage: S,
    repository: R,
    ttl: u32,

    #[builder(setter(skip))]
    #[new(default)]
    inflight: Arc<InflightMap>,
}

struct InflightGuard<'a> {
    key: &'a str,
    inflight: &'a InflightMap,
    notify: Arc<Notify>,
}

impl Drop for InflightGuard<'_> {
    fn drop(&mut self) {
        let mut map = self.inflight.lock();
        map.remove(self.key);
        self.notify.notify_waiters();
    }
}

impl<S: Storage, R> Cache<S, R> {
    pub async fn load_or_cache<T: Serialize + DeserializeOwned + Sync, E>(
        &self,
        key: impl AsRef<str>,
        load: impl AsyncFnOnce(&R) -> Result<T, E>,
    ) -> Result<T, E> {
        let key = key.as_ref();

        loop {
            if let Some(cached) = self.fetch_cache(key).await {
                return Ok(cached);
            }

            trace!("Reached cache miss for {}", type_name::<T>());

            let (is_leader, notify) = {
                let mut in_flight = self.inflight.lock();
                if let Some(notify) = in_flight.get(key) {
                    (false, notify.clone())
                } else {
                    let notify = Arc::new(Notify::new());
                    in_flight.insert(key.to_owned() , notify.clone());
                    (true, notify)
                }
            };

            if !is_leader {
                notify.notified().await;
                continue
            }

            let _guard = InflightGuard {
                key,
                inflight: &self.inflight,
                notify
            };

            let result = load(&self.repository).await?;
            self.cache(key, &result).await;
            drop(_guard);

            return Ok(result)
        }
    }

    async fn cache<T: Serialize + Sync>(&self, key: &str, value: &T) {
        _ = self.storage
            .set(key, value, Some(self.ttl))
            .await
            .inspect_err(|e| error!("failed to cache item in storage: {e}"));
    }

    #[inline(always)]
    async fn fetch_cache<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.storage
            .get::<_, T>(key)
            .await
            .inspect_err(|e| error!("Failed to fetch cached item from storage: {e}"))
            .ok()
            .flatten()
    }

    pub async fn clear_cache_keys<K, I>(&self, keys: I)
    where
        K: Into<String> + Send,
        I: IntoIterator<Item = K> + Send,
        I::IntoIter: Send
    {
        _ = self.storage.bulk_delete(keys).await
            .inspect_err(|e| error!("Failed to bulk delete keys: {e}"));
    }

    pub fn repository(&self) -> &R {
         &self.repository
    }
}
