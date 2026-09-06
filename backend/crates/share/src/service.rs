use service::service;
use futures::Stream;
use nutype::nutype;
use thiserror::Error;
use uuid::Uuid;
use domain::models::folders;

pub enum ShareEvent {

}

#[derive(Debug, Error)]
pub enum InitiateSessionError {

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


}
