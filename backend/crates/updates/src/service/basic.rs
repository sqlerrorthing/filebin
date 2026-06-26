use crate::service::{FolderUpdate, UpdatesService};
use derive_new::new;
use domain::entity::{files, folders};
use futures::Stream;
use futures::StreamExt;
use parking_lot::Mutex;
use pin_project::pin_project;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

#[pin_project]
#[derive(new)]
struct DebugStream<S> {
    #[pin]
    inner: S,
}

impl<S> Debug for DebugStream<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("DebugStream")
    }
}

impl<S: Stream> Stream for DebugStream<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }
}

#[derive(Debug, new)]
pub struct BasicUpdatesService {
    #[new(default)]
    folders_channels: Mutex<HashMap<folders::Id, broadcast::Sender<Arc<FolderUpdate>>>>,
    channel_capacity: usize,
}

impl BasicUpdatesService {
    fn get_or_create_folders_channel(
        &self,
        folder_id: folders::Id,
    ) -> broadcast::Sender<Arc<FolderUpdate>> {
        let mut map = self.folders_channels.lock();
        if let Some(sender) = map.get(&folder_id) {
            return sender.clone();
        }

        let (sender, _) = broadcast::channel(self.channel_capacity);
        map.insert(folder_id, sender.clone());
        sender
    }

    fn get_channel(&self, folder_id: folders::Id) -> Option<broadcast::Sender<Arc<FolderUpdate>>> {
        self.folders_channels.lock().get(&folder_id).cloned()
    }

    fn send_update(&self, folder_id: folders::Id, update: FolderUpdate) {
        if let Some(sender) = self.get_channel(folder_id) {
            _ = sender.send(Arc::new(update))
        }
    }

    fn folder_deleted(&self, folder_id: folders::Id) {
        let mut map = self.folders_channels.lock();
        map.remove(&folder_id);
    }
}

impl UpdatesService for BasicUpdatesService {
    type FoldersUpdateStream = impl Stream<Item = Arc<FolderUpdate>> + Debug;

    fn subscribe_folder(&self, folder_id: folders::Id) -> Self::FoldersUpdateStream {
        let sender = self.get_or_create_folders_channel(folder_id);
        let receiver = sender.subscribe();

        DebugStream::new(BroadcastStream::new(receiver).filter_map(|res| async move { res.ok() }))
    }

    fn file_uploaded(&self, file: files::Model) {
        self.send_update(file.folder_id, FolderUpdate::FileUploaded(file))
    }

    fn folder_renamed(&self, folder_id: folders::Id, new_folder_name: String) {
        self.send_update(
            folder_id,
            FolderUpdate::FolderRenamed((folder_id, new_folder_name)),
        )
    }

    fn folder_deleted(&self, folder: folders::Model) {
        let id = folder.id;
        self.send_update(id, FolderUpdate::FolderDeleted(folder));
        self.folder_deleted(id);
    }
}
