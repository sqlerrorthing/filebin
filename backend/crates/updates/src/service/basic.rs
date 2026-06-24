use std::fmt::Debug;
use std::sync::Arc;
use futures::Stream;
use domain::entity::{files, folders};
use crate::service::{Update, UpdatesService};

#[derive(Debug)]
pub struct BasicUpdatesService {
    
}

impl UpdatesService for BasicUpdatesService {
    type FoldersUpdateStream = impl Stream<Item = Arc<Update>> + Debug;

    fn subscribe_folder(&self, folder_id: folders::Id) -> Self::FoldersUpdateStream {
        todo!()
    }

    fn file_uploaded(&self, file: files::Model) {
        todo!()
    }

    fn folder_renamed(&self, folder_id: folders::Id, new_folder_name: String) {
        todo!()
    }
}