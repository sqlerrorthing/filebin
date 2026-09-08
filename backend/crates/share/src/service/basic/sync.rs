pub mod basic;
pub mod rabbitmq;

use service::service;
use futures::Stream;
use std::fmt::Debug;
use crate::service::{SessionId, ShareEvent};

#[service(dynamic)]
pub trait ShareSyncService {
    type ShareStream: Stream<Item = ShareEvent> + Send + Sync + Debug + 'static;

    fn subscribe_session(&self, session_id: SessionId) -> Self::ShareStream;
    fn broadcast_event(&self, session_id: SessionId, event: ShareEvent);
    fn session_created(&self, session_id: SessionId);
    fn session_closed(&self, session_id: SessionId);
    fn stop_rotation(&self, session_id: SessionId);
}
