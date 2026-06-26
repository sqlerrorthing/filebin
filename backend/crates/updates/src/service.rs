pub mod basic;
pub mod rabbitmq;

use domain::entity::{files, folders};
use futures::Stream;
use service::service;
use std::sync::Arc;
use derive_new::new;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, new)]
pub struct FolderUpdate {
    pub folder_id: folders::Id,
    pub kind: FolderUpdateKind
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FolderUpdateKind {
    FileUploaded(files::Model),
    FolderRenamed(String),
    FolderDeleted(folders::Model)
}

#[service(dynamic)]
pub trait UpdatesService {
    /// The stream will close when the folder is deleted.
    /// This stream can be dropped to unsubscribe
    type FoldersUpdateStream: Stream<Item = Arc<FolderUpdate>>;

    fn subscribe_folder(&self, folder_id: folders::Id) -> Self::FoldersUpdateStream;

    fn file_uploaded(&self, file: files::Model);

    fn folder_renamed(&self, folder_id: folders::Id, new_folder_name: String);

    fn folder_deleted(&self, folder: folders::Model);
}
