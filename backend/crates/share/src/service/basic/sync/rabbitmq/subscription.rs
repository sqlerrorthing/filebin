use crate::service::SessionId;
use crate::service::basic::sync::rabbitmq::{BindingCmd, BindingCmdKind};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{Level, debug, span};

#[derive(Debug)]
pub struct SubscriptionGuard {
    pub(in crate::service) session_id: SessionId,
    pub(in crate::service) binding_tx: UnboundedSender<BindingCmd>,
    pub(in crate::service) counts: Arc<Mutex<HashMap<SessionId, usize>>>,
}

impl Drop for SubscriptionGuard {
    fn drop(&mut self) {
        let _span = span!(Level::DEBUG, "drop session subscription guard", session_id = %self.session_id).entered();

        let mut counts = self.counts.lock();
        if let Some(count) = counts.get_mut(&self.session_id) {
            *count -= 1;
            debug!("active count is {count}");
            if *count == 0 {
                debug!("dropping full session subscription cause no one needed it");
                counts.remove(&self.session_id);
                let _ = self.binding_tx.send(BindingCmd::new(self.session_id, BindingCmdKind::Unbind));
            }
        }
    }
}
