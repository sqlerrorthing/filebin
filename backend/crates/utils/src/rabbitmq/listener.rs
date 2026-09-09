use crate::rabbitmq::message::{Message, SessionId};
use std::fmt::Debug;
use std::marker::PhantomData;

pub trait Listener: Send + Sync + Clone + 'static {
    type SessionId: SessionId;
    type LocalSessionData: LocalSessionData<Listener = Self>;

    type Message: Message;
    type StreamItem: From<Self::Message> + Clone + Send + Sync;

    fn on_message(&self, _session_id: &Self::SessionId, _message: Self::Message) {}
}

pub trait LocalSessionData: Debug + Clone {
    type Listener: Listener;

    fn new(session_id: &<Self::Listener as Listener>::SessionId) -> Self;
}

impl<L: Listener> LocalSessionData for PhantomData<L> {
    type Listener = L;

    fn new(_session_id: &<Self::Listener as Listener>::SessionId) -> Self {
        PhantomData
    }
}
