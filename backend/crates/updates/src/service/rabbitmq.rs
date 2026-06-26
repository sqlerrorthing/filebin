pub mod stream;
pub mod subscription;

use std::collections::HashMap;
use amqprs::consumer::AsyncConsumer;
use crate::service::{FolderUpdate, FolderUpdateKind, UpdatesService};
use amqprs::connection::Connection;
use derivative::Derivative;
use domain::entity::{files, folders};
use futures::Stream;
use std::sync::Arc;
use amqprs::{BasicProperties, Deliver};
use amqprs::channel::{BasicConsumeArguments, BasicPublishArguments, Channel, QueueBindArguments, QueueDeclareArguments, QueueUnbindArguments};
use parking_lot::Mutex;
use tokio::spawn;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::{debug, span, Level};
use service::async_trait::async_trait;
use crate::service::basic::LocalUpdatesService;
use crate::service::rabbitmq::stream::SubscriptionGuardStream;
use crate::service::rabbitmq::subscription::SubscriptionGuard;

struct PublishCmd {
    routing_key: String,
    payload: Vec<u8>,
}

enum BindingCmd {
    Bind(folders::Id),
    Unbind(folders::Id),
}

struct InstanceRabbitMQConsumer {
    local_service: Arc<LocalUpdatesService>,
    counts: Arc<Mutex<HashMap<folders::Id, usize>>>,
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
        if let Ok(update) = postcard::from_bytes::<FolderUpdate>(&content) {
            let folder_id = update.folder_id;
            let is_delete = matches!(update.kind, FolderUpdateKind::FolderDeleted(_));
            self.local_service.send_update(folder_id, update.kind);

            if is_delete {
                let mut counts = self.counts.lock();
                if counts.remove(&folder_id).is_some() {
                    let _ = self.binding_tx.send(BindingCmd::Unbind(folder_id));
                }
            }
        }
    }
}

/// Uses RabbitMQ to publish updates
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct RabbitMQUpdatesService {
    exchange: String,
    #[derivative(Debug = "ignore")]
    publish_tx: UnboundedSender<PublishCmd>,
    #[derivative(Debug = "ignore")]
    binding_tx: UnboundedSender<BindingCmd>,
    #[derivative(Debug = "ignore")]
    counts: Arc<Mutex<HashMap<folders::Id, usize>>>,
    #[derivative(Debug = "ignore")]
    local_service: Arc<LocalUpdatesService>,
}

fn get_routing_key(folder_id: folders::Id) -> String {
    format!("folder.{}", folder_id)
}

impl RabbitMQUpdatesService {
    pub fn new(exchange: String, connection: Connection, local_updates_service: LocalUpdatesService) -> Self {
        let local_updates_service = Arc::new(local_updates_service);
        let connection = Arc::new(connection);
        let (publish_tx, mut publish_rx) = mpsc::unbounded_channel::<PublishCmd>();
        let (binding_tx, mut binding_rx) = mpsc::unbounded_channel::<BindingCmd>();
        let counts = Arc::new(Mutex::new(HashMap::new()));

        let conn_publish = connection.clone();
        let exchange_publish = exchange.clone();
        spawn(async move {
            if let Ok(channel) = conn_publish.open_channel(None).await {
                while let Some(cmd) = publish_rx.recv().await {
                    let args = BasicPublishArguments::new(&exchange_publish, &cmd.routing_key);
                    let _ = channel.basic_publish(BasicProperties::default(), cmd.payload, args).await;
                }
            }
        });

        let conn_consume = connection.clone();
        let exchange_consume = exchange.clone();
        let local_service_cloned = local_updates_service.clone();

        let binding_tx_clone = binding_tx.clone();
        let counts_clone = counts.clone();
        spawn(async move {
            if let Ok(channel) = conn_consume.open_channel(None).await {
                let queue_args = QueueDeclareArguments::default()
                    .exclusive(true)
                    .auto_delete(true)
                    .finish();

                if let Ok(Some((queue_name, _, _))) = channel.queue_declare(queue_args).await {
                    let consume_args = BasicConsumeArguments::new(&queue_name, "")
                        .manual_ack(false)
                        .finish();

                    let consumer = InstanceRabbitMQConsumer { local_service: local_service_cloned, binding_tx: binding_tx_clone, counts: counts_clone };

                    if channel.basic_consume(consumer, consume_args).await.is_ok() {
                        while let Some(cmd) = binding_rx.recv().await {
                            match cmd {
                                BindingCmd::Bind(id) => {
                                    let routing_key = get_routing_key(id);
                                    let args = QueueBindArguments::new(&queue_name, &exchange_consume, &routing_key);
                                    let _ = channel.queue_bind(args).await;
                                }
                                BindingCmd::Unbind(id) => {
                                    let routing_key = get_routing_key(id);
                                    let args = QueueUnbindArguments::new(&queue_name, &exchange_consume, &routing_key);
                                    let _ = channel.queue_unbind(args).await;
                                }
                            }
                        }
                    }
                }
            }
        });

        Self { publish_tx, binding_tx, counts, exchange, local_service: local_updates_service }
    }

    fn send_update(&self, folder_id: folders::Id, kind: FolderUpdateKind) {
        let routing_key = get_routing_key(folder_id);
        let update = FolderUpdate {
            folder_id,
            kind
        };

        if let Ok(payload) = postcard::to_allocvec(&update) {
            _ = self.publish_tx.send(PublishCmd { routing_key, payload })
        }
    }
}

impl UpdatesService for RabbitMQUpdatesService {
    type FoldersUpdateStream = impl Stream<Item = Arc<FolderUpdate>>;

    fn subscribe_folder(&self, folder_id: folders::Id) -> Self::FoldersUpdateStream {
        let _span = span!(Level::DEBUG, "subscribing folder", %folder_id).entered();
        let mut counts = self.counts.lock();

        let count = counts.entry(folder_id).or_insert(0);
        if *count == 0 {
            debug!("binding folder");
            let _ = self.binding_tx.send(BindingCmd::Bind(folder_id));
        } else {
            debug!(count = count, "this folder are already bound");
        }
        *count += 1;
        drop(counts);
        let inner_stream = self.local_service.subscribe_folder(folder_id);

        SubscriptionGuardStream {
            inner: inner_stream,
            _guard: SubscriptionGuard {
                folder_id,
                binding_tx: self.binding_tx.clone(),
                counts: self.counts.clone(),
            }
        }
    }

    fn file_uploaded(&self, file: files::Model) {
        self.send_update(file.folder_id, FolderUpdateKind::FileUploaded(file));
    }

    fn folder_renamed(&self, folder_id: folders::Id, new_folder_name: String)  {
        self.send_update(folder_id, FolderUpdateKind::FolderRenamed(new_folder_name))
    }

    fn folder_deleted(&self, folder: folders::Model) {
        self.send_update(folder.id, FolderUpdateKind::FolderDeleted(folder))
    }
}
