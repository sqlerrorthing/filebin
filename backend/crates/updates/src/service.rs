pub mod basic;

use domain::entity::{files, folders};
use futures::Stream;
use service::service;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum FolderUpdate {
    FileUploaded(files::Model),
    FolderRenamed((folders::Id, String)),
    FolderDeleted(folders::Model)
}

#[service]
pub trait UpdatesService: 'static {
    /// The stream will close when the folder is deleted.
    /// This stream can be dropped to unsubscribe
    type FoldersUpdateStream: Stream<Item = Arc<FolderUpdate>>;

    fn subscribe_folder(&self, folder_id: folders::Id) -> Self::FoldersUpdateStream;

    fn file_uploaded(&self, file: files::Model);

    fn folder_renamed(&self, folder_id: folders::Id, new_folder_name: String);

    fn folder_deleted(&self, folder: folders::Model);
}
