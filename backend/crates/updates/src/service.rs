pub mod basic;

use domain::entity::{files, folders};
use futures::Stream;
use service::service;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Update {
    FileUploaded(files::Model),
}

#[service]
pub trait UpdatesService {
    /// The stream will close when the folder is deleted.
    type FoldersUpdateStream: Stream<Item = Arc<Update>>;

    fn subscribe_folder(&self, folder_id: folders::Id) -> Self::FoldersUpdateStream;

    fn file_uploaded(&self, file: files::Model);

    fn folder_renamed(&self, folder_id: folders::Id, new_folder_name: String);
}
