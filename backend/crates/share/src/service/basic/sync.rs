pub mod basic;
pub mod rabbitmq;

use service::{service, map};
use futures::Stream;
use std::fmt::Debug;
use tokio_util::sync::CancellationToken;
use crate::service::{SessionId, ShareEvent};

pub struct SubscribedSession<S> {
    pub(crate) stream: S,
    pub(crate) code_rotate_cancel: CancellationToken
}

impl<S> SubscribedSession<S> {
    pub fn map<N>(self, f: impl FnOnce(S) -> N) -> SubscribedSession<N> {
        SubscribedSession {
            stream: f(self.stream),
            code_rotate_cancel: self.code_rotate_cancel,
        }
    }
}

#[service(dynamic)]
pub trait ShareSyncService {
    type ShareStream: Stream<Item = ShareEvent> + Send + Sync + Debug + 'static;

    #[map(SubscribedSession::map)]
    fn subscribe_session(&self, session_id: SessionId) -> SubscribedSession<Self::ShareStream>;
    fn broadcast_event(&self, session_id: SessionId, event: ShareEvent);
    fn session_created(&self, session_id: SessionId);
    fn session_closed(&self, session_id: SessionId);
    fn stop_rotation(&self, session_id: SessionId);
}
