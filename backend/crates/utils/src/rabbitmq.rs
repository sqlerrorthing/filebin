use crate::rabbitmq::consumer::{BindingCmd, BindingCmdKind, InstanceRabbitMQConsumer};
use crate::rabbitmq::listener::Listener;
use crate::rabbitmq::message::{PublishCmd, SessionId};
use amqprs::BasicProperties;
use amqprs::channel::{
    BasicConsumeArguments, BasicPublishArguments, Channel, ExchangeDeclareArguments,
    QueueBindArguments, QueueDeclareArguments, QueueUnbindArguments,
};
use amqprs::connection::Connection;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use tokio::spawn;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{Level, error, info, span};

pub mod consumer;
mod listener;
pub mod message;

pub struct RabbitMQSync<L: Listener> {
    listener: L,
    routing_prefix: Cow<'static, str>,
    publish_tx: UnboundedSender<PublishCmd>,
    binding_tx: UnboundedSender<BindingCmd<L::SessionId>>,
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
                            let _span = span!(Level::DEBUG, "queue bind/unbind queue", folder_id = %session_id, action = ?cmd.kind, routing_key = %routing_key);

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
            publish_tx,
            binding_tx,
            listener,
            routing_prefix,
        }
    }

}
