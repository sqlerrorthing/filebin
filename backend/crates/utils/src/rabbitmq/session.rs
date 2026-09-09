use crate::rabbitmq::listener::{Listener, LocalSessionData};
use crate::stream::DebugStream;
use derivative::Derivative;
use derive_new::new;
use futures::{Stream, StreamExt};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

#[derive(new)]
pub struct SubscribedSession<S, L>
where
    S: Stream<Item = L::StreamItem>,
    L: Listener,
{
    pub session: S,
    pub data: L::LocalSessionData,
    marker: PhantomData<L>,
}

impl<S, L> SubscribedSession<S, L>
where
    L: Listener,
    S: Stream<Item = L::StreamItem>,
{
    pub(crate) fn map_session<N>(self, f: impl FnOnce(S) -> N) -> SubscribedSession<N, L>
    where
        N: Stream<Item = L::StreamItem>,
    {
        SubscribedSession {
            session: f(self.session),
            data: self.data,
            marker: PhantomData,
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug, Clone(bound = ""))]
pub(super) struct SessionControl<L: Listener> {
    pub(super) tx: broadcast::Sender<L::StreamItem>,
    pub(super) data: L::LocalSessionData,
}

#[derive(Derivative, new)]
#[derivative(Debug(bound = ""), Clone(bound = ""))]
pub(super) struct LocalSessions<L: Listener> {
    #[new(default)]
    sessions: Arc<Mutex<HashMap<L::SessionId, SessionControl<L>>>>,
}

impl<L: Listener> LocalSessions<L> {
    pub fn get_or_create_sender(
        &self,
        session_id: &L::SessionId,
    ) -> broadcast::Sender<L::StreamItem> {
        let mut map = self.sessions.lock();

        if let Some(control) = map.get(session_id) {
            return control.tx.clone();
        }

        let (tx, _) = broadcast::channel(32);

        map.insert(
            session_id.clone(),
            SessionControl {
                tx: tx.clone(),
                data: L::LocalSessionData::new(session_id),
            },
        );

        tx
    }

    pub fn get_or_create_sender_with_control(
        &self,
        session_id: &L::SessionId,
    ) -> (broadcast::Sender<L::StreamItem>, L::LocalSessionData) {
        let mut map = self.sessions.lock();

        if let Some(control) = map.get(session_id) {
            return (control.tx.clone(), control.data.clone());
        }

        let (tx, _) = broadcast::channel(32);
        let data = L::LocalSessionData::new(session_id);

        let control = SessionControl {
            tx: tx.clone(),
            data: data.clone(),
        };

        map.insert(session_id.clone(), control);

        (tx, data)
    }

    pub fn remove_session(&self, session_id: &L::SessionId) {
        self.sessions.lock().remove(session_id);
    }

    pub fn subscribe_session(
        &self,
        session_id: &L::SessionId,
    ) -> SubscribedSession<impl Stream<Item = L::StreamItem> + Send + Sync + Debug + use<L>, L>
    {
        let (tx, data) = self.get_or_create_sender_with_control(session_id);
        let rx = tx.subscribe();

        SubscribedSession::new(
            DebugStream::new(BroadcastStream::new(rx).filter_map(|res| async move { res.ok() })),
            data,
        )
    }

    pub fn broadcast_message(&self, session_id: &L::SessionId, message: L::Message) {
        _ = self.get_or_create_sender(session_id).send(message.into());
    }

    pub fn close(&self, session_id: &L::SessionId) {
        self.sessions.lock().remove(session_id);
    }

    pub fn with_session_control<R>(
        &self,
        session_id: &L::SessionId,
        f: impl FnOnce(&SessionControl<L>) -> R,
    ) -> Option<R> {
        self.sessions
            .lock()
            .get(session_id)
            .map(move |sess| f(sess))
    }
}
