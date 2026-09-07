pub mod stream;
pub mod subscription;

use crate::service::sync::basic::LocalShareSyncService;
use crate::service::sync::ShareSyncService;
use crate::service::sync::rabbitmq::stream::SubscriptionGuardStream;
use crate::service::sync::rabbitmq::subscription::SubscriptionGuard;
use crate::service::{SessionId, ShareEvent};
use amqprs::channel::{
    BasicConsumeArguments, BasicPublishArguments, Channel, ExchangeDeclareArguments,
    QueueBindArguments, QueueDeclareArguments, QueueUnbindArguments,
};
use amqprs::connection::Connection;
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use derivative::Derivative;
use futures::Stream;
use parking_lot::Mutex;
use service::async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use derive_new::new;
use serde::{Deserialize, Serialize};
use tokio::spawn;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, error, info, span, Level};

struct PublishCmd {
    routing_key: String,
    payload: Vec<u8>,
}

#[derive(Debug, new)]
pub struct BindingCmd {
    pub session_id: SessionId,
    pub kind: BindingCmdKind,
}

#[derive(Debug, Clone, Copy)]
pub enum BindingCmdKind {
    Bind,
    Unbind,
}

#[derive(Serialize, Deserialize)]
struct ShareRabbitMessage {
    session_id: SessionId,
    event: ShareEvent,
}

struct InstanceRabbitMQConsumer {
    local_service: Arc<LocalShareSyncService>,
    counts: Arc<Mutex<HashMap<SessionId, usize>>>,
    binding_tx: UnboundedSender<BindingCmd>,
}

#[async_trait]
impl AsyncConsumer for InstanceRabbitMQConsumer {
    async fn consume(
        &mut self,
        _channel: &Channel,
        _deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        if let Ok(msg) = postcard::from_bytes::<ShareRabbitMessage>(&content) {
            let session_id = msg.session_id;
            let is_close = matches!(msg.event, ShareEvent::SessionClosed);
            self.local_service.broadcast_event(session_id, msg.event);

            if is_close {
                let mut counts = self.counts.lock();
                if counts.remove(&session_id).is_some() {
                    let _ = self.binding_tx.send(BindingCmd::new(session_id, BindingCmdKind::Unbind));
                }
            }
        }
    }
}

/// Uses RabbitMQ to publish and synchronize share session events
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct RabbitMQShareSyncService {
    exchange: String,
    #[derivative(Debug = "ignore")]
    publish_tx: UnboundedSender<PublishCmd>,
    #[derivative(Debug = "ignore")]
    binding_tx: UnboundedSender<BindingCmd>,
    #[derivative(Debug = "ignore")]
    counts: Arc<Mutex<HashMap<SessionId, usize>>>,
    #[derivative(Debug = "ignore")]
    local_service: Arc<LocalShareSyncService>,
}

fn get_routing_key(session_id: SessionId) -> String {
    format!("session.{}", session_id)
}

async fn declare_exchange(channel: &Channel, exchange: &str) {
    if let Err(err) = channel
        .exchange_declare(
            ExchangeDeclareArguments::new(exchange, "topic")
                .durable(true)
                .finish(),
        )
        .await
    {
        error!(err = %err, "fail to declare exchange (share publish)");
    }
}

impl RabbitMQShareSyncService {
    pub fn new(
        exchange: String,
        connection: Connection,
        local_share_sync_service: LocalShareSyncService,
    ) -> Self {
        let local_share_sync_service = Arc::new(local_share_sync_service);
        let connection = Arc::new(connection);
        let (publish_tx, mut publish_rx) = mpsc::unbounded_channel::<PublishCmd>();
        let (binding_tx, mut binding_rx) = mpsc::unbounded_channel::<BindingCmd>();
        let counts = Arc::new(Mutex::new(HashMap::new()));

        let conn_publish = connection.clone();
        let exchange_publish = exchange.clone();
        spawn(async move {
            if let Ok(channel) = conn_publish.open_channel(None).await {
                declare_exchange(&channel, &exchange_publish).await;

                while let Some(cmd) = publish_rx.recv().await {
                    let _ = channel
                        .basic_publish(
                            BasicProperties::default(),
                            cmd.payload,
                            BasicPublishArguments::new(&exchange_publish, &cmd.routing_key),
                        )
                        .await;
                }
            }
        });

        let conn_consume = connection.clone();
        let exchange_consume = exchange.clone();
        let local_service_cloned = local_share_sync_service.clone();
        let binding_tx_clone = binding_tx.clone();
        let counts_clone = counts.clone();

        spawn(async move {
            if let Ok(channel) = conn_consume.open_channel(None).await {
                declare_exchange(&channel, &exchange_consume).await;

                let queue_args = QueueDeclareArguments::default()
                    .exclusive(true)
                    .auto_delete(true)
                    .finish();

                if let Ok(Some((queue_name, _, _))) = channel.queue_declare(queue_args).await {
                    let consume_args = BasicConsumeArguments::new(&queue_name, "")
                        .manual_ack(false)
                        .finish();

                    let consumer = InstanceRabbitMQConsumer {
                        local_service: local_service_cloned,
                        binding_tx: binding_tx_clone,
                        counts: counts_clone,
                    };

                    if channel.basic_consume(consumer, consume_args).await.is_ok() {
                        while let Some(cmd) = binding_rx.recv().await {
                            let session_id = cmd.session_id;
                            let routing_key = get_routing_key(session_id);
                            let _span = span!(Level::DEBUG, "queue bind/unbind session queue", session_id = %session_id, action = ?cmd.kind, routing_key = %routing_key);

                            match cmd.kind {
                                BindingCmdKind::Bind => {
                                    let args = QueueBindArguments::new(
                                        &queue_name,
                                        &exchange_consume,
                                        &routing_key,
                                    );
                                    if let Err(err) = channel.queue_bind(args).await {
                                        error!(err = %err, "failed to bind queue");
                                    } else {
                                        info!("bound session successfully");
                                    }
                                }
                                BindingCmdKind::Unbind => {
                                    let args = QueueUnbindArguments::new(
                                        &queue_name,
                                        &exchange_consume,
                                        &routing_key,
                                    );
                                    if let Err(err) = channel.queue_unbind(args).await {
                                        error!(err = %err, "failed to unbind queue");
                                    } else {
                                        info!("unbound session successfully");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Self {
            exchange,
            publish_tx,
            binding_tx,
            counts,
            local_service: local_share_sync_service,
        }
    }
}

impl ShareSyncService for RabbitMQShareSyncService {
    type ShareStream = SubscriptionGuardStream<<LocalShareSyncService as ShareSyncService>::ShareStream>;

    fn subscribe_session(&self, session_id: SessionId) -> Self::ShareStream {
        let _span = span!(Level::DEBUG, "subscribing session", %session_id).entered();
        let mut counts = self.counts.lock();

        let count = counts.entry(session_id).or_insert(0);
        if *count == 0 {
            debug!("binding session");
            let _ = self.binding_tx.send(BindingCmd::new(session_id, BindingCmdKind::Bind));
        } else {
            debug!(count = count, "this session is already bound");
        }
        *count += 1;
        drop(counts);

        let inner_stream = self.local_service.subscribe_session(session_id);

        SubscriptionGuardStream {
            inner: inner_stream,
            _guard: SubscriptionGuard {
                session_id,
                binding_tx: self.binding_tx.clone(),
                counts: self.counts.clone(),
            },
        }
    }

    fn broadcast_event(&self, session_id: SessionId, event: ShareEvent) {
        self.local_service.broadcast_event(session_id, event.clone());

        let routing_key = get_routing_key(session_id);
        let msg = ShareRabbitMessage { session_id, event };
        if let Ok(payload) = postcard::to_allocvec(&msg) {
            _ = self.publish_tx.send(PublishCmd {
                routing_key,
                payload,
            });
        }
    }

    fn session_created(&self, session_id: SessionId) {
        self.local_service.session_created(session_id);
    }

    fn session_closed(&self, session_id: SessionId) {
        let mut counts = self.counts.lock();
        if counts.remove(&session_id).is_some() {
            let _ = self.binding_tx.send(BindingCmd::new(session_id, BindingCmdKind::Unbind));
        }
        self.local_service.session_closed(session_id);
    }

    fn stop_rotation(&self, session_id: SessionId) {
        self.local_service.stop_rotation(session_id);
    }

    fn set_cancel_tx(&self, session_id: SessionId, cancel_tx: tokio::sync::watch::Sender<()>) {
        self.local_service.set_cancel_tx(session_id, cancel_tx);
    }
}
