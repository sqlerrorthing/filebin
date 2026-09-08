use std::fmt::Display;
use std::hash::Hash;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

pub(super) struct PublishCmd {
    pub(crate) routing_key: String,
    pub(crate) payload: Vec<u8>
}

// todo: use trait aliases
pub trait SessionId: Serialize + DeserializeOwned + Hash + Eq + PartialEq + Send + Sync + Clone + Display + 'static {}
impl<S: Serialize + DeserializeOwned + Hash + Eq + PartialEq + Send + Sync + Clone + Display + 'static> SessionId for S {}

pub trait Message: Serialize + DeserializeOwned + 'static {
    type SessionId: SessionId;

    fn session_id(&self) -> Self::SessionId;

    fn is_close(&self) -> bool;
}
