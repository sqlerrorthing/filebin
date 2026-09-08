use crate::service::basic::sync::ShareSyncService;
use crate::service::{SessionId, ShareEvent};
use amqprs::connection::Connection;
use amqprs::consumer::AsyncConsumer;
use derivative::Derivative;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use utils::rabbitmq::listener::{Listener, LocalSessionControl};
use utils::rabbitmq::message::Message;
use utils::rabbitmq::RabbitMQSync;

struct PublishCmd {
    routing_key: String,
    payload: Vec<u8>,
}

/// Uses RabbitMQ to publish and synchronize share session events
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct RabbitMQShareSyncService {
    inner: RabbitMQSync<SyncListener>
}

struct SyncListener;

impl Listener for SyncListener {
    type SessionId = SessionId;
    type LocalSessionControl = SessionControl;
    type Message = ShareEvent;
    type StreamItem = Arc<ShareEvent>;

    fn on_message(&self, session_id: &Self::SessionId, message: Self::Message) {
        dbg!((session_id, message));
    }
}

impl Message for ShareEvent {
    fn is_close(&self) -> bool {
        matches!(self, ShareEvent::SessionClosed)
    }
}

struct SessionControl {
    cancel_code_rotation: CancellationToken
}

impl LocalSessionControl for SessionControl {
    type Listener = SyncListener;

    fn new(_session_id: &<Self::Listener as Listener>::SessionId) -> Self {
        Self {
            cancel_code_rotation: CancellationToken::new()
        }
    }
}

impl RabbitMQShareSyncService {
    pub fn new(
        exchange: String,
        connection: Connection,
    ) -> Self {
        Self {
            inner: RabbitMQSync::new(
                exchange,
                connection,
                "share",
                SyncListener
            )
        }
    }
}

impl RabbitMQShareSyncService {
    pub fn stop_rotation(&self, session_id: &SessionId) {
        self.inner.with_local_session_control(session_id, |sess| {
            sess.cancel_code_rotation.cancel()
        });
    }
}

impl ShareSyncService for RabbitMQShareSyncService {
    type ShareStream = impl Stream<Item = ShareEvent> + Send + Sync + Debug + 'static;

    fn subscribe_session(&self, session_id: SessionId) -> Self::ShareStream {
        self.inner.subscribe_session(session_id)
            .map(|s| s.to_owned())
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
