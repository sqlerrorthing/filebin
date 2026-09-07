use crate::service::sync::ShareSyncService;
use crate::service::{SessionId, ShareEvent};
use derive_new::new;
use futures::Stream;
use futures::StreamExt;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use utils::stream::DebugStream;

#[derive(Debug, Clone)]
pub struct SessionControl {
    pub tx: broadcast::Sender<ShareEvent>,
    pub cancel_tx: tokio::sync::watch::Sender<()>,
}

#[derive(Debug, Clone, new)]
pub struct LocalShareSyncService {
    #[new(default)]
    sessions: Arc<Mutex<HashMap<SessionId, SessionControl>>>,
}

impl LocalShareSyncService {
    pub fn get_or_create_sender(&self, session_id: SessionId) -> broadcast::Sender<ShareEvent> {
        let mut map = self.sessions.lock();
        if let Some(control) = map.get(&session_id) {
            return control.tx.clone();
        }
        let (tx, _) = broadcast::channel(32);
        let (cancel_tx, _) = tokio::sync::watch::channel(());
        map.insert(
            session_id,
            SessionControl {
                tx: tx.clone(),
                cancel_tx,
            },
        );
        tx
    }

    pub fn set_cancel_tx(&self, session_id: SessionId, cancel_tx: tokio::sync::watch::Sender<()>) {
        let mut map = self.sessions.lock();
        if let Some(control) = map.get_mut(&session_id) {
            control.cancel_tx = cancel_tx;
        } else {
            let (tx, _) = broadcast::channel(32);
            map.insert(
                session_id,
                SessionControl {
                    tx,
                    cancel_tx,
                },
            );
        }
    }

    pub fn stop_rotation(&self, session_id: SessionId) {
        let map = self.sessions.lock();
        if let Some(control) = map.get(&session_id) {
            _ = control.cancel_tx.send(());
        }
    }

    pub fn remove_session(&self, session_id: &SessionId) {
        let mut map = self.sessions.lock();
        map.remove(session_id);
    }
}

impl ShareSyncService for LocalShareSyncService {
    type ShareStream = DebugStream<impl Stream<Item = ShareEvent> + Send + Sync + 'static>;

    fn subscribe_session(&self, session_id: SessionId) -> Self::ShareStream {
        let tx = self.get_or_create_sender(session_id);
        let rx = tx.subscribe();

        DebugStream::new(BroadcastStream::new(rx).filter_map(|res| async move { res.ok() }))
    }

    fn broadcast_event(&self, session_id: SessionId, event: ShareEvent) {
        if let Some(control) = self.sessions.lock().get(&session_id) {
            _ = control.tx.send(event);
        }
    }

    fn session_created(&self, session_id: SessionId) {
        let _ = self.get_or_create_sender(session_id);
    }

    fn session_closed(&self, session_id: SessionId) {
        self.stop_rotation(session_id);
        self.remove_session(&session_id);
    }

    fn stop_rotation(&self, session_id: SessionId) {
        LocalShareSyncService::stop_rotation(self, session_id);
    }

    fn set_cancel_tx(&self, session_id: SessionId, cancel_tx: tokio::sync::watch::Sender<()>) {
        LocalShareSyncService::set_cancel_tx(self, session_id, cancel_tx);
    }
}
