pub mod basic;

use bytes::Bytes;
use service::service;
use futures::Stream;
use nutype::nutype;
use thiserror::Error;
use uuid::Uuid;
use domain::models::folders;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

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
    SessionFailed {
        error_message: String,
    },
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

#[nutype(derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Display, From, FromStr, Serialize, Deserialize))]
pub struct SessionId(Uuid);

#[nutype(derive(Debug, Clone, PartialEq, Eq, Display, From, FromStr, Serialize, Deserialize, AsRef))]
pub struct Code(String);

#[derive(Debug, Error)]
pub enum InitiateSessionError {
    #[error("folder not found")]
    FolderNotFound,
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum JoinSessionError {
    #[error("invalid code")]
    InvalidCode,
    #[error("session not found")]
    SessionNotFound,
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum SendKeyError {
    #[error("session not found")]
    SessionNotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("internal error: {0}")]
    Internal(String),
}

#[service(dynamic)]
pub trait ShareService {
    type Error;
    type ShareStream: Stream<Item = ShareEvent> + Send + 'static;

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
    async fn send_key(
        &self,
        session_id: SessionId,
        folder_key: Bytes,
    );
}
