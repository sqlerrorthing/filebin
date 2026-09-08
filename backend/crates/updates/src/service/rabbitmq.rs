use crate::service::{FolderUpdate, FolderUpdateKind, UpdatesService};
use amqprs::connection::Connection;
use domain::models::{files, folders};
use futures::Stream;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use utils::rabbitmq::listener::Listener;
use utils::rabbitmq::message::Message;
use utils::rabbitmq::RabbitMQSync;

struct PublishCmd {
    routing_key: String,
    payload: Vec<u8>,
}

/// Uses RabbitMQ to publish updates
#[derive(Debug, Clone)]
pub struct RabbitMQUpdatesService {
    inner: RabbitMQSync<SyncListener>
}

#[derive(Debug, Clone)]
struct SyncListener;

impl Listener for SyncListener {
    type SessionId = folders::Id;
    type LocalSessionControl = PhantomData<Self>;
    type Message = FolderUpdate;
    type StreamItem = Arc<FolderUpdate>;
}

impl Message for FolderUpdate {
    fn is_close(&self) -> bool {
        matches!(self.kind, FolderUpdateKind::FolderDeleted { .. })
    }
}

impl RabbitMQUpdatesService {
    pub fn new(
        exchange: String,
        connection: Connection,
    ) -> Self {
        Self {
            inner: RabbitMQSync::new(
                exchange,
                connection,
                "folders",
                SyncListener
            ),
        }
    }

    fn send_update(&self, folder_id: folders::Id, kind: FolderUpdateKind) {
        self.inner.broadcast_message(folder_id, FolderUpdate {
            folder_id,
            kind
        });
    }
}

impl UpdatesService for RabbitMQUpdatesService {
    type FoldersUpdateStream = impl Stream<Item = Arc<FolderUpdate>> + Debug + 'static;

    fn subscribe_folder(&self, folder_id: folders::Id) -> Self::FoldersUpdateStream {
        self.inner.subscribe_session(folder_id)
    }

    fn fire_file_uploaded(&self, file: files::Model) {
        self.send_update(file.folder_id, FolderUpdateKind::FileUploaded { file });
    }

    fn fire_file_deleted(&self, file: files::Model) -> () {
        self.send_update(file.folder_id, FolderUpdateKind::FileDeleted { file })
    }

    fn fire_folder_renamed(&self, folder_id: folders::Id, new_folder_name: folders::FolderName) {
        self.send_update(folder_id, FolderUpdateKind::FolderRenamed { new_folder_name })
    }

    fn fire_folder_deleted(&self, folder: folders::Model) {
        self.send_update(folder.id, FolderUpdateKind::FolderDeleted { folder })
    }
}
