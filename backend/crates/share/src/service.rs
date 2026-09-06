use bytes::Bytes;
use service::service;
use futures::Stream;
use nutype::nutype;
use thiserror::Error;
use uuid::Uuid;
use domain::models::folders;

pub enum ShareEvent {

}

macro_rules! keys {
    ($($key:ident)*) => {
        $(
            #[nutype(
                derive(Debug, TryFrom, Serialize, Deserialize),
                validate(predicate = |b| b.len() == 32)
            )]
            pub struct $key(bytes::Bytes);
        )*
    };
}

keys!(SenderPublicKey ReceiverPublicKey);

#[nutype(derive(Debug, Display, From, FromStr, Serialize, Deserialize))]
pub struct SessionId(Uuid);

#[nutype(derive(Debug, Display, From, FromStr, Serialize, Deserialize))]
pub struct Code(String);


#[derive(Debug, Error)]
pub enum InitiateSessionError {

}

#[derive(Debug, Error)]
pub enum JoinSessionError {

}

#[derive(Debug, Error)]
pub enum SendKeyError {

}

#[service(dynamic)]
pub trait ShareService {
    type Error;
    type ShareStream: Stream<Item = ShareEvent>;

    #[result(InitiateSessionError)]
    async fn initiate_session(
        &self,
        folder_id: folders::Id,
        public_key: SenderPublicKey
    ) -> (SessionId, Self::ShareStream);

    #[result(JoinSessionError)]
    async fn join_session(
        &self,
        code: Code,
        public_key: ReceiverPublicKey
    ) -> (SessionId, Self::ShareStream);

    #[result(SendKeyError)]
    async fn send_key(
        &self,
        session_id: SessionId,
        folder_key: Bytes
    );
}
