use crate::service::basic::BasicShareService;
use crate::service::basic::sync::ShareSyncService;
use crate::service::{SessionId, ShareEvent};
use folders::service::FoldersService;
use futures::Stream;
use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;
use std::task::{Context, Poll};
use storage::Storage;
use tokio::spawn;

#[pin_project(PinnedDrop)]
pub struct SessionStream<St, S, FS, SS>
where
    S: Storage,
    FS: FoldersService + Clone,
    SS: ShareSyncService + Clone,
{
    #[pin]
    pub inner: St,
    pub service: BasicShareService<S, FS, SS>,
    pub session_id: SessionId,
}

impl<St, S, FS, SS> Stream for SessionStream<St, S, FS, SS>
where
    St: Stream<Item = ShareEvent>,
    S: Storage,
    FS: FoldersService + Clone,
    SS: ShareSyncService + Clone,
{
    type Item = ShareEvent;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }
}

#[pinned_drop]
impl<St, S, FS, SS> PinnedDrop for SessionStream<St, S, FS, SS>
where
    S: Storage,
    FS: FoldersService + Clone,
    SS: ShareSyncService + Clone,
{
    fn drop(self: Pin<&mut Self>) {
        let service = self.service.clone();
        let session_id = self.session_id;
        spawn(async move {
            _ = service.cancel_session_internal(&session_id).await;
        });
    }
}
