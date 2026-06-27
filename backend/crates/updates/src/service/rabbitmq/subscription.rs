use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, span, Level};
use domain::entity::folders;
use crate::service::rabbitmq::{BindingCmd, BindingCmdKind};

pub struct SubscriptionGuard {
    pub(super) folder_id: folders::Id,
    pub(super) binding_tx: UnboundedSender<BindingCmd>,
    pub(super) counts: Arc<Mutex<HashMap<folders::Id, usize>>>,
}

impl Drop for SubscriptionGuard {
    fn drop(&mut self) {
        let _span = span!(Level::DEBUG, "drop subscription guard", folder_id = %self.folder_id).entered();

        let mut counts = self.counts.lock();
        if let Some(count) = counts.get_mut(&self.folder_id) {
            *count -= 1;
            debug!("active count is {count}");
            if *count == 0 {
                debug!("dropping full folder subscription cause no one needed it");
                counts.remove(&self.folder_id);
                let _ = self.binding_tx.send(BindingCmd::new(self.folder_id, BindingCmdKind::Unbind));
            }
        }
    }
}
