use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::task::{Context, Poll};
use derive_new::new;
use futures::Stream;
use pin_project::pin_project;

#[pin_project]
#[derive(new)]
pub struct DebugStream<S> {
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
