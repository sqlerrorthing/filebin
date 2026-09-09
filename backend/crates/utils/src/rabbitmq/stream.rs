use crate::rabbitmq::Debug;
use crate::rabbitmq::consumer::{BindingCmd, BindingCmdKind};
use crate::rabbitmq::listener::Listener;
use derivative::Derivative;
use derive_new::new;
use futures::Stream;
use parking_lot::Mutex;
use pin_project::{pin_project, pinned_drop};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, debug_span};

#[derive(Derivative, new)]
#[derivative(Debug(bound = "S: Debug"))]
#[pin_project(PinnedDrop)]
pub struct SubscriptionGuardStream<S, L: Listener> {
    #[pin]
    inner: S,
    session_id: L::SessionId,
    binding_tx: UnboundedSender<BindingCmd<L::SessionId>>,
    counts: Arc<Mutex<HashMap<L::SessionId, usize>>>,
}

#[pinned_drop]
impl<S, L: Listener> PinnedDrop for SubscriptionGuardStream<S, L> {
    fn drop(self: Pin<&mut Self>) {
        let _span = debug_span!("drop subscription guard", session_id = %self.session_id).entered();

        let mut counts = self.counts.lock();
        if let Some(count) = counts.get_mut(&self.session_id) {
            *count -= 1;
            debug!("active count is {count}");
            if *count == 0 {
                debug!("dropping full folder subscription cause no one needed it");
                counts.remove(&self.session_id);
                let _ = self.binding_tx.send(BindingCmd::new(
                    self.session_id.clone(),
                    BindingCmdKind::Unbind,
                ));
            }
        }
    }
}

impl<S, I, L: Listener> Stream for SubscriptionGuardStream<S, L>
where
    S: Stream<Item = I>,
{
    type Item = I;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.poll_next(cx)
    }
}
