use crate::rabbitmq::consumer::{
    BindingCmd, BindingCmdKind, InstanceRabbitMQConsumer, QueueMessage,
};
use crate::rabbitmq::listener::Listener;
use crate::rabbitmq::message::{PublishCmd, SessionId};
use crate::rabbitmq::session::{LocalSessions, SubscribedSession};
use crate::rabbitmq::stream::SubscriptionGuardStream;
use amqprs::BasicProperties;
use amqprs::channel::{
    BasicConsumeArguments, BasicPublishArguments, Channel, ExchangeDeclareArguments,
    QueueBindArguments, QueueDeclareArguments, QueueUnbindArguments,
};
use amqprs::connection::Connection;
use futures::Stream;
use parking_lot::Mutex;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{Level, debug, debug_span, error, info, span};

pub mod consumer;
pub mod listener;
pub mod message;
pub mod session;
pub mod stream;

#[derive(Debug, Clone)]
pub struct RabbitMQSync<L: Listener> {
    local: LocalSessions<L>,
    routing_prefix: Cow<'static, str>,
    publish_tx: UnboundedSender<PublishCmd>,
    binding_tx: UnboundedSender<BindingCmd<L::SessionId>>,
    counts: Arc<Mutex<HashMap<L::SessionId, usize>>>,
}

async fn declare_exchange(channel: &Channel, exchange: &str) -> Result<(), amqprs::error::Error> {
    channel
        .exchange_declare(
            ExchangeDeclareArguments::new(exchange, "topic")
                .durable(true)
                .finish(),
        )
        .await
}

macro_rules! on_err {
    ($expr:expr, $msg:expr) => {
        if let Err(err) = $expr {
            error!(err = %err, $msg)
        }
    };
}

fn normalize_routing_prefix(s: Cow<str>) -> Cow<str> {
    if let Some(stripped) = s.strip_suffix('.') {
        Cow::Owned(stripped.to_string())
    } else {
        s
    }
}

fn routing_key(prefix: &str, id: &impl SessionId) -> String {
    format!("{prefix}.{id}")
}

impl<L: Listener> RabbitMQSync<L> {
    pub fn new(
        exchange: String,
        connection: Connection,
        routing_prefix: impl Into<Cow<'static, str>>,
        listener: L,
    ) -> Self {
        let routing_prefix = normalize_routing_prefix(routing_prefix.into());
        let connection = Arc::new(connection);

        let (publish_tx, mut publish_rx) = mpsc::unbounded_channel::<PublishCmd>();
        let (binding_tx, mut binding_rx) = mpsc::unbounded_channel::<BindingCmd<L::SessionId>>();

        let counts: Arc<Mutex<HashMap<_, usize>>> = Default::default();

        let conn_publish = connection.clone();
        let exchange_publish = exchange.clone();

        spawn(async move {
            if let Ok(channel) = conn_publish.open_channel(None).await {
                on_err!(
                    declare_exchange(&channel, &exchange_publish).await,
                    "fail to declare exchange (publish)"
                );

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
        let listener_clone = listener.clone();

        let binding_tx_clone = binding_tx.clone();
        let counts_clone = counts.clone();

        let routing_prefix_clone = routing_prefix.clone();

        spawn(async move {
            if let Ok(channel) = conn_consume.open_channel(None).await {
                on_err!(
                    declare_exchange(&channel, &exchange_consume).await,
                    "fail to declare exchange (consume)"
                );

                let queue_args = QueueDeclareArguments::default()
                    .exclusive(true)
                    .auto_delete(true)
                    .finish();

                if let Ok(Some((queue_name, _, _))) = channel.queue_declare(queue_args).await {
                    let consume_args = BasicConsumeArguments::new(&queue_name, "")
                        .manual_ack(false)
                        .finish();

                    let consumer = InstanceRabbitMQConsumer::new(
                        listener_clone,
                        counts_clone,
                        binding_tx_clone,
                    );

                    let routing_prefix = routing_prefix_clone.as_ref();

                    if channel.basic_consume(consumer, consume_args).await.is_ok() {
                        while let Some(cmd) = binding_rx.recv().await {
                            let session_id = cmd.session_id;
                            let routing_key = routing_key(routing_prefix, &session_id);
                            let _span = debug_span!("queue bind/unbind queue", folder_id = %session_id, action = ?cmd.kind, routing_key = %routing_key);

                            match cmd.kind {
                                BindingCmdKind::Bind => {
                                    let args = QueueBindArguments::new(
                                        &queue_name,
                                        &exchange_consume,
                                        &routing_key,
                                    );
                                    if let Err(err) = channel.queue_bind(args).await {
                                        error!(err = %err, "failed to bound queue");
                                    } else {
                                        info!("bound successfully");
                                    }
                                }
                                BindingCmdKind::Unbind => {
                                    let args = QueueUnbindArguments::new(
                                        &queue_name,
                                        &exchange_consume,
                                        &routing_key,
                                    );

                                    if let Err(err) = channel.queue_unbind(args).await {
                                        error!(err = %err, "failed to unbound queue {routing_prefix}");
                                    } else {
                                        info!("unbound {routing_prefix} successfully");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Self {
            local: LocalSessions::new(),
            publish_tx,
            binding_tx,
            routing_prefix,
            counts,
        }
    }

    pub fn subscribe_session(
        &self,
        session_id: L::SessionId,
    ) -> SubscribedSession<impl Stream<Item = L::StreamItem> + Debug + use<L>, L> {
        let _span = debug_span!("subscribing session", %session_id).entered();

        {
            let mut counts = self.counts.lock();
            let count = counts.entry(session_id.clone()).or_insert(0);
            if *count == 0 {
                debug!("binding session");
                _ = self
                    .binding_tx
                    .send(BindingCmd::new(session_id.clone(), BindingCmdKind::Bind));
            } else {
                debug!(count = count, "this session is already bound")
            }

            *count += 1;
        }

        let inner = self.local.subscribe_session(&session_id);

        inner.map_session(|inner| {
            SubscriptionGuardStream::<_, L>::new(
                inner,
                session_id,
                self.binding_tx.clone(),
                self.counts.clone(),
            )
        })
    }

    pub fn with_local_session_control<R>(
        &self,
        session_id: &L::SessionId,
        f: impl FnOnce(&L::LocalSessionData) -> R,
    ) -> Option<R> {
        self.local
            .with_session_control(session_id, move |sess| f(&sess.data))
    }

    pub fn broadcast_message(&self, session_id: L::SessionId, message: L::Message) {
        self.local.broadcast_message(&session_id, message.clone());

        let routing_key = routing_key(self.routing_prefix.as_ref(), &session_id);
        let msg = QueueMessage::<L> {
            session_id,
            message,
        };

        _ = self.publish_tx.send(PublishCmd {
            routing_key,
            payload: postcard::to_allocvec(&msg).expect("valid serialized postcard message"),
        })
    }

    pub fn close(&self, session_id: L::SessionId) {
        if self.counts.lock().remove(&session_id).is_some() {
            _ = self
                .binding_tx
                .send(BindingCmd::new(session_id.clone(), BindingCmdKind::Unbind));
        }

        self.local.close(&session_id);
    }
}
