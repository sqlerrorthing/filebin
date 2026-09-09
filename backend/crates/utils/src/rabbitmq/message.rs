use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::{Debug, Display};
use std::hash::Hash;

pub(super) struct PublishCmd {
    pub(crate) routing_key: String,
    pub(crate) payload: Vec<u8>,
}

// todo: use trait aliases
pub trait SessionId:
    Debug
    + Serialize
    + DeserializeOwned
    + Hash
    + Eq
    + PartialEq
    + Send
    + Sync
    + Clone
    + Display
    + 'static
{
}
impl<
    S: Serialize
        + Debug
        + DeserializeOwned
        + Hash
        + Eq
        + PartialEq
        + Send
        + Sync
        + Clone
        + Display
        + 'static,
> SessionId for S
{
}

pub trait Message: Serialize + DeserializeOwned + Clone + 'static {
    fn is_close(&self) -> bool;
}
