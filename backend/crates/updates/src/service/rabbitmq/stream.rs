use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Stream;
use pin_project::pin_project;
use crate::service::rabbitmq::subscription::SubscriptionGuard;

#[pin_project]
pub struct SubscriptionGuardStream<S> {
    #[pin]
    pub(super) inner: S,
    pub(super) _guard: SubscriptionGuard
}

impl<S, I> Stream for SubscriptionGuardStream<S>
where
    S: Stream<Item = I>
{
    type Item = I;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.poll_next(cx)
    }
}
