pub mod basic;

pub use basic::sync::DynShareSyncService;
pub use basic::sync::ShareSyncService;

use bytes::Bytes;
use domain::models::folders;
use futures::Stream;
use nutype::nutype;
use serde::{Deserialize, Serialize};
use service::service;
use std::fmt::Debug;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShareEvent {
    CodeRotated {
        code: Code,
        ttl_seconds: i32,
    },
    ReceiverJoined {
        receiver_public_key: ReceiverPublicKey,
    },
    SessionConnected {
        sender_public_key: SenderPublicKey,
    },
    KeyReceived {
        encrypted_folder_key: Bytes,
        folder_public_id: folders::PublicId,
    },
    SessionClosed,
}

macro_rules! keys {
    ($($key:ident)*) => {
        $(
            #[nutype(
                derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TryFrom),
                validate(predicate = |b| b.len() == 32)
            )]
            pub struct $key(bytes::Bytes);
        )*
    };
}

keys!(SenderPublicKey ReceiverPublicKey);

#[nutype(derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Display,
    From,
    FromStr,
    Serialize,
    Deserialize
))]
pub struct SessionId(Uuid);

#[nutype(derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Display,
    From,
    FromStr,
    Serialize,
    Deserialize,
    AsRef
))]
pub struct Code(String);

#[derive(Debug, Error)]
pub enum InitiateSessionError {
    #[error("folder not found")]
    FolderNotFound,
}

#[derive(Debug, Error)]
pub enum JoinSessionError {
    #[error("invalid code")]
    InvalidCode,
    #[error("session not found")]
    SessionNotFound,
}

#[derive(Debug, Error)]
pub enum SendKeyError {
    #[error("session not found")]
    SessionNotFound,
    #[error("unauthorized")]
    Unauthorized,
}

#[derive(Debug, Error)]
pub enum CancelSessionError {
    #[error("session not found")]
    SessionNotFound,
}

#[service(dynamic)]
pub trait ShareService {
    type Error;
    type ShareStream: Stream<Item = ShareEvent> + 'static;

    #[result(InitiateSessionError)]
    async fn initiate_session(
        &self,
        folder_id: folders::PublicId,
        public_key: SenderPublicKey,
    ) -> (SessionId, Self::ShareStream);

    #[result(JoinSessionError)]
    async fn join_session(
        &self,
        code: Code,
        public_key: ReceiverPublicKey,
    ) -> (SessionId, Self::ShareStream);

    #[result(SendKeyError)]
    async fn send_key(&self, session_id: SessionId, folder_key: Bytes);

    #[result(CancelSessionError)]
    async fn cancel_session(&self, session_id: SessionId);
}
