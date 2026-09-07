use std::pin::Pin;
use std::task::{Context, Poll};
use pin_project::{pin_project, pinned_drop};
use folders::service::FoldersService;
use futures::Stream;
use tokio::spawn;
use storage::Storage;
use crate::service::basic::BasicLocalShareService;
use crate::service::{SessionId, ShareEvent};

#[pin_project(PinnedDrop)]
pub(in super) struct SessionStream<St, S, FS>
where
    S: Storage,
    FS: FoldersService + Clone,
{
    #[pin]
    pub(in super) inner: St,
    pub(in super) service: BasicLocalShareService<S, FS>,
    pub(in super) session_id: SessionId,
}

impl<St, S, FS> Stream for SessionStream<St, S, FS>
where
    St: Stream<Item = ShareEvent>,
    S: Storage,
    FS: FoldersService + Clone,
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
impl<St, S, FS> PinnedDrop for SessionStream<St, S, FS>
where
    S: Storage,
    FS: FoldersService + Clone,
{
    fn drop(self: Pin<&mut Self>) {
        let service = self.service.clone();
        let session_id = self.session_id;
        spawn(async move {
            _ = service.cancel_session_internal(&session_id).await;
        });
    }
}
