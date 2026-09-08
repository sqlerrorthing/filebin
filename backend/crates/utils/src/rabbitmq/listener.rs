use crate::rabbitmq::message::{Message, SessionId};

pub trait Listener: Send + Sync + Clone + 'static {
    type SessionId: SessionId;
    type Message: Message<SessionId = Self::SessionId>;

    fn on_message(&self, session_id: &Self::SessionId, message: Self::Message);
}