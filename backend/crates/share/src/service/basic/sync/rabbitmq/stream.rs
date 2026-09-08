use crate::service::basic::sync::rabbitmq::subscription::SubscriptionGuard;
use futures::Stream;
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
#[pin_project]
pub struct SubscriptionGuardStream<S> {
    #[pin]
    pub(in crate::service) inner: S,
    pub(in crate::service) _guard: SubscriptionGuard,
}

impl<S, I> Stream for SubscriptionGuardStream<S>
where
    S: Stream<Item = I>,
{
    type Item = I;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.poll_next(cx)
    }
}
