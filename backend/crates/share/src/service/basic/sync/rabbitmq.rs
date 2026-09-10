use crate::service::basic::sync::{ShareSyncService, SubscribedSession};
use crate::service::{SessionId, ShareEvent};
use amqprs::connection::Connection;
use derivative::Derivative;
use futures::Stream;
use std::fmt::Debug;
use tokio_util::sync::CancellationToken;
use utils::rabbitmq::RabbitMQSync;
use utils::rabbitmq::listener::{Listener, LocalSessionData};
use utils::rabbitmq::message::Message;

/// Uses RabbitMQ to publish and synchronize share session events
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct RabbitMQShareSyncService {
    inner: RabbitMQSync<SyncListener>,
}

#[derive(Debug, Clone)]
struct SyncListener;

impl Listener for SyncListener {
    type SessionId = SessionId;
    type LocalSessionData = SessionControl;
    type Message = ShareEvent;
    type StreamItem = ShareEvent;
}

impl Message for ShareEvent {
    fn is_close(&self) -> bool {
        matches!(self, ShareEvent::SessionClosed)
    }
}

#[derive(Clone, Debug)]
struct SessionControl {
    cancel_code_rotation: CancellationToken,
}

impl LocalSessionData for SessionControl {
    type Listener = SyncListener;

    fn new(_session_id: &<Self::Listener as Listener>::SessionId) -> Self {
        Self {
            cancel_code_rotation: CancellationToken::new(),
        }
    }
}

impl RabbitMQShareSyncService {
    pub fn new(exchange: String, connection: Connection) -> Self {
        Self {
            inner: RabbitMQSync::new(exchange, connection, "share", SyncListener),
        }
    }
}

impl RabbitMQShareSyncService {
    pub fn stop_rotation(&self, session_id: &SessionId) {
        self.inner
            .with_local_session_control(session_id, |sess| sess.cancel_code_rotation.cancel());
    }
}

impl ShareSyncService for RabbitMQShareSyncService {
    type ShareStream = impl Stream<Item = ShareEvent> + Send + Sync + Debug + 'static;

    fn subscribe_session(&self, session_id: SessionId) -> SubscribedSession<Self::ShareStream> {
        let sess = self.inner.subscribe_session(session_id);

        SubscribedSession {
            stream: sess.session,
            code_rotate_cancel: sess.data.cancel_code_rotation,
        }
    }

    fn broadcast_event(&self, session_id: SessionId, event: ShareEvent) {
        self.inner.broadcast_message(session_id, event);
    }

    fn session_created(&self, _session_id: SessionId) {
        /* noop */
    }

    fn session_closed(&self, session_id: SessionId) {
        RabbitMQShareSyncService::stop_rotation(self, &session_id);
        self.inner.close(session_id);
    }

    fn stop_rotation(&self, session_id: SessionId) {
        RabbitMQShareSyncService::stop_rotation(self, &session_id);
    }
}
