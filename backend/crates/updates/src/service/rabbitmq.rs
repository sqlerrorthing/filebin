use crate::service::{FolderUpdate, UpdatesService};
use amqprs::connection::Connection;
use derivative::Derivative;
use derive_new::new;
use domain::entity::{files, folders};
use futures::{Stream, stream};
use std::sync::Arc;

/// Uses RabbitMQ to publish updates
#[derive(Derivative, Clone, new)]
#[derivative(Debug)]
pub struct RabbitMQUpdatesService {
    #[derivative(Debug = "ignore")]
    connection: Connection,
}

impl UpdatesService for RabbitMQUpdatesService {
    type FoldersUpdateStream = impl Stream<Item = Arc<FolderUpdate>>;

    fn subscribe_folder(&self, folder_id: folders::Id) -> Self::FoldersUpdateStream {
        stream::empty()
    }

    fn file_uploaded(&self, file: files::Model) -> () {
        todo!()
    }

    fn folder_renamed(&self, folder_id: folders::Id, new_folder_name: String) -> () {
        todo!()
    }

    fn folder_deleted(&self, folder: folders::Model) -> () {
        todo!()
    }
}
