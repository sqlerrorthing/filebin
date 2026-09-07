pub mod stream;

use crate::service::basic::stream::SessionStream;
use crate::service::{
    CancelSessionError, Code, InitiateSessionError, JoinSessionError, ReceiverPublicKey,
    SendKeyError, SenderPublicKey, SessionId, ShareEvent, ShareService,
};
use bytes::Bytes;
use derive_new::new;
use domain::models;
use folders::service::FoldersService;
use futures::Stream;
use futures::StreamExt;
use parking_lot::Mutex;
use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};
use service::business;
use service::error::ServiceError;
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use storage::Storage;
use thiserror::Error;
use tokio::spawn;
use tokio::sync::{broadcast, watch};
use tokio::time::sleep;
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: SessionId,
    pub folder_id: models::folders::PublicId,
    pub sender_public_key: SenderPublicKey,
    pub receiver_public_key: Option<ReceiverPublicKey>,
    pub code: Code,
}

#[derive(Debug, Clone)]
pub struct SessionControl {
    pub tx: broadcast::Sender<ShareEvent>,
    pub cancel_tx: watch::Sender<()>,
}

#[derive(Debug, Error)]
pub enum Error<S, FS>
where
    S: Storage,
    FS: FoldersService,
{
    #[error("storage error: {0}")]
    Storage(#[source] S::Error),
    #[error("folders service error: {0}")]
    Folders(#[source] FS::Error),
}

#[derive(Debug, Clone, new)]
pub struct BasicLocalShareService<S, FS> {
    storage: S,
    folders_service: FS,
    code_ttl: Duration,
    #[new(default)]
    sessions: Arc<Mutex<HashMap<SessionId, SessionControl>>>,
}

fn generate_code() -> Code {
    let mut rng = rand::rng();

    // todo: replace with this alphabet
    // let alphabet = "234679ACDEFGHJKMNPQRTUVWXY";

    let code: u32 = rng.random_range(100000..=999999);
    code.to_string().into()
}

fn session_key(session_id: &SessionId) -> String {
    format!("share:session:{session_id}")
}

fn code_key(code: &Code) -> String {
    format!("share:code:{code}")
}

impl<S, FS> BasicLocalShareService<S, FS>
where
    S: Storage,
    FS: FoldersService,
{
    pub fn get_or_create_sender(&self, session_id: SessionId) -> broadcast::Sender<ShareEvent> {
        let mut sessions = self.sessions.lock();
        if let Some(control) = sessions.get(&session_id) {
            return control.tx.clone();
        }
        let (tx, _) = broadcast::channel(32);
        let (cancel_tx, _) = watch::channel(());
        sessions.insert(
            session_id,
            SessionControl {
                tx: tx.clone(),
                cancel_tx,
            },
        );
        tx
    }

    pub fn stop_rotation(&self, session_id: &SessionId) {
        if let Some(control) = self.sessions.lock().get(session_id) {
            _ = control.cancel_tx.send(());
        }
    }

    pub fn broadcast_event(&self, session_id: SessionId, event: ShareEvent) {
        if let Some(control) = self.sessions.lock().get(&session_id) {
            _ = control.tx.send(event);
        }
    }

    async fn save_session(&self, state: &SessionState) -> Result<(), Error<S, FS>> {
        self.storage
            .set(
                &session_key(&state.session_id),
                state,
                3600,
            )
            .await
            .map_err(Error::Storage)
    }

    async fn get_session(
        &self,
        session_id: &SessionId,
    ) -> Result<Option<SessionState>, Error<S, FS>> {
        self.storage
            .get(&session_key(session_id))
            .await
            .map_err(Error::Storage)
    }

    async fn set_code_mapping(
        &self,
        code: &Code,
        session_id: SessionId,
    ) -> Result<(), Error<S, FS>> {
        self.storage
            .set(
                &code_key(code),
                &session_id.to_string(),
                self.code_ttl.as_secs() as u32,
            )
            .await
            .map_err(Error::Storage)
    }

    async fn get_session_id_by_code(&self, code: &Code) -> Result<Option<SessionId>, Error<S, FS>> {
        let id_str: Option<String> = self
            .storage
            .get(&code_key(code))
            .await
            .map_err(Error::Storage)?;
        Ok(id_str.and_then(|s| SessionId::from_str(&s).ok()))
    }

    async fn delete_code_mapping(&self, code: &Code) -> Result<(), Error<S, FS>> {
        _ = self.storage.delete(&code_key(code)).await;
        Ok(())
    }

    async fn cancel_session_internal(&self, session_id: &SessionId) -> Result<(), Error<S, FS>> {
        let session = match self.get_session(session_id).await? {
            Some(s) => s,
            None => {
                self.stop_rotation(session_id);
                self.sessions.lock().remove(session_id);
                return Ok(());
            }
        };

        _ = self.delete_code_mapping(&session.code).await;
        self.stop_rotation(session_id);
        self.storage
            .delete(&session_key(session_id))
            .await
            .map_err(Error::Storage)?;

        self.broadcast_event(*session_id, ShareEvent::SessionClosed);

        self.sessions.lock().remove(session_id);

        Ok(())
    }
}

fn broadcast_strean<T: Clone + Send + 'static>(
    rx: broadcast::Receiver<T>,
) -> impl Stream<Item = T> + Send + 'static {
    BroadcastStream::new(rx).filter_map(|res| async move { res.ok() })
}

impl<S, FS> ShareService for BasicLocalShareService<S, FS>
where
    S: Storage,
    FS: FoldersService + Clone,
{
    type Error = Error<S, FS>;
    type ShareStream = impl Stream<Item = ShareEvent> + Send + 'static;

    async fn initiate_session(
        &self,
        folder_id: models::folders::PublicId,
        public_key: SenderPublicKey,
    ) -> Result<(SessionId, Self::ShareStream), ServiceError<InitiateSessionError, Self::Error>>
    {
        let folder = match self
            .folders_service
            .find_folder_by_public_id(folder_id)
            .await
            .map_err(Error::Folders)?
        {
            Some(f) => f,
            None => return Err(business!(InitiateSessionError::FolderNotFound)),
        };

        let session_id = Uuid::now_v7().into();
        let code = generate_code();

        let state = SessionState {
            session_id,
            folder_id: folder.public_id,
            sender_public_key: public_key,
            receiver_public_key: None,
            code: code.clone(),
        };

        self.save_session(&state).await?;
        self.set_code_mapping(&code, session_id).await?;

        let tx = self.get_or_create_sender(session_id);
        let rx = tx.subscribe();

        let (cancel_tx, mut cancel_rx) = watch::channel(());
        {
            let mut sessions = self.sessions.lock();
            if let Some(control) = sessions.get_mut(&session_id) {
                control.cancel_tx = cancel_tx;
            } else {
                sessions.insert(
                    session_id,
                    SessionControl {
                        tx: tx.clone(),
                        cancel_tx,
                    },
                );
            }
        }

        let ttl_secs = self.code_ttl.as_secs() as i32;
        _ = tx.send(ShareEvent::CodeRotated {
            code: code.clone(),
            ttl_seconds: ttl_secs,
        });

        let this = self.clone();
        let session_id_cloned = session_id;
        let code_ttl = self.code_ttl;
        spawn(async move {
            let mut current_code = code;
            loop {
                tokio::select! {
                    _ = sleep(code_ttl) => {
                        let mut session = match this.get_session(&session_id_cloned).await {
                            Ok(Some(s)) => s,
                            _ => break,
                        };

                        if session.receiver_public_key.is_some() {
                            break;
                        }

                        _ = this.delete_code_mapping(&current_code).await;

                        let new_code = generate_code();
                        session.code = new_code.clone();

                        if this.save_session(&session).await.is_err() {
                            break;
                        }
                        if this
                            .set_code_mapping(&new_code, session_id_cloned)
                            .await
                            .is_err()
                        {
                            break;
                        }

                        current_code = new_code.clone();
                        this.broadcast_event(
                            session_id_cloned,
                            ShareEvent::CodeRotated {
                                code: new_code,
                                ttl_seconds: code_ttl.as_secs() as i32,
                            },
                        );
                    }
                    _ = cancel_rx.changed() => {
                        break;
                    }
                }
            }
        });

        let stream = SessionStream {
            inner: broadcast_strean(rx),
            service: self.clone(),
            session_id,
        };

        Ok((session_id, stream))
    }

    async fn join_session(
        &self,
        code: Code,
        receiver_pk: ReceiverPublicKey,
    ) -> Result<(SessionId, Self::ShareStream), ServiceError<JoinSessionError, Self::Error>> {
        let Some(session_id) = self.get_session_id_by_code(&code).await? else {
            return Err(business!(JoinSessionError::InvalidCode));
        };

        let Some(mut session) = self.get_session(&session_id).await? else {
            return Err(business!(JoinSessionError::SessionNotFound));
        };

        _ = self.delete_code_mapping(&code).await;
        _ = self.delete_code_mapping(&session.code).await;
        self.stop_rotation(&session_id);

        let sender_pk = session.sender_public_key.clone();

        session.receiver_public_key = Some(receiver_pk.clone());
        self.save_session(&session).await?;

        let tx = self.get_or_create_sender(session_id);
        let rx = tx.subscribe();

        _ = tx.send(ShareEvent::ReceiverJoined {
            receiver_public_key: receiver_pk,
        });

        _ = tx.send(ShareEvent::SessionConnected {
            sender_public_key: sender_pk,
        });

        let stream = SessionStream {
            inner: broadcast_strean(rx),
            service: self.clone(),
            session_id,
        };

        Ok((session_id, stream))
    }

    async fn send_key(
        &self,
        session_id: SessionId,
        folder_key: Bytes,
    ) -> Result<(), ServiceError<SendKeyError, Self::Error>> {
        let Some(session) = self.get_session(&session_id).await? else {
            return Err(business!(SendKeyError::SessionNotFound));
        };

        self.broadcast_event(
            session_id,
            ShareEvent::KeyReceived {
                encrypted_folder_key: folder_key,
                folder_public_id: session.folder_id,
            },
        );

        Ok(())
    }

    async fn cancel_session(
        &self,
        session_id: SessionId,
    ) -> Result<(), ServiceError<CancelSessionError, Self::Error>> {
        let Some(session) = self.get_session(&session_id).await? else {
            return Err(business!(CancelSessionError::SessionNotFound));
        };

        self.cancel_session_internal(&session.session_id).await?;
        Ok(())
    }
}
