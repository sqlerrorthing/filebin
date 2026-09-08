use crate::rabbitmq::listener::Listener;
use crate::rabbitmq::message::Message;
use amqprs::BasicProperties;
use amqprs::Deliver;
use amqprs::channel::Channel;
use amqprs::consumer::AsyncConsumer;
use async_trait::async_trait;
use derive_new::new;
use futures::SinkExt;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

#[derive(new)]
pub struct InstanceRabbitMQConsumer<L: Listener> {
    listener: L,
    counts: Arc<Mutex<HashMap<L::SessionId, usize>>>,
    binding_tx: UnboundedSender<BindingCmd<L::SessionId>>,
}

#[derive(Debug, new, Serialize, Deserialize)]
pub struct BindingCmd<Id> {
    pub(super) session_id: Id,
    pub(super) kind: BindingCmdKind
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingCmdKind {
    Bind,
    Unbind
}

#[async_trait]
impl<L> AsyncConsumer for InstanceRabbitMQConsumer<L>
where
    L: Listener,
{
    async fn consume(
        &mut self,
        _channel: &Channel,
        _deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        if let Ok(update) = postcard::from_bytes::<L::Message>(&content) {
            let session_id = update.session_id();
            let is_close = update.is_close();

            self.listener.on_message(&session_id, update);

            if is_close {
                let mut counts = self.counts.lock();
                if counts.remove(&session_id).is_some() {
                    let _ = self.binding_tx.send(BindingCmd::new(session_id, BindingCmdKind::Unbind));
                }
            }
        }
    }
}
