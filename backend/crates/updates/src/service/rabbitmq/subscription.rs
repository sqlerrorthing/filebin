use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use tokio::sync::mpsc::UnboundedSender;
use domain::entity::folders;
use crate::service::rabbitmq::BindingCmd;

pub struct SubscriptionGuard {
    pub(super) folder_id: folders::Id,
    pub(super) binding_tx: UnboundedSender<BindingCmd>,
    pub(super) counts: Arc<Mutex<HashMap<folders::Id, usize>>>,
}

impl Drop for SubscriptionGuard {
    fn drop(&mut self) {
        let mut counts = self.counts.lock();
        if let Some(count) = counts.get_mut(&self.folder_id) {
            *count -= 1;
            if *count == 0 {
                counts.remove(&self.folder_id);
                let _ = self.binding_tx.send(BindingCmd::Unbind(self.folder_id));
            }
        }
    }
}
