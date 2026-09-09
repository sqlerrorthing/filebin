use crate::schema::SplitBusinessResultExt;
use crate::schema::api::folder::v1::share_service_server::ShareService as GrpcShareService;
use crate::schema::api::folder::v1::{
    JoinRequest, SendKeyRequest, SendKeyResponse, ShareEvent as ProtoShareEvent, ShareRequest,
};
use async_trait::async_trait;
use derive_new::new;
use domain::models;
use futures::Stream;
use futures::StreamExt;
use share::service::{
    Code, ReceiverPublicKey, SendKeyError, SenderPublicKey, SessionId, ShareService,
};
use std::str::FromStr;
use tonic::{Request, Response, Status};

#[derive(new)]
#[allow(clippy::redundant_field_names)]
pub struct BasicGrpcShareService<SS> {
    share_service: SS,
}

#[async_trait]
impl<SS> GrpcShareService for BasicGrpcShareService<SS>
where
    SS: ShareService,
{
    type ShareStream = impl Stream<Item = Result<ProtoShareEvent, Status>> + Send + 'static;

    async fn share(
        &self,
        request: Request<ShareRequest>,
    ) -> Result<Response<Self::ShareStream>, Status> {
        let payload = request.into_inner();
        let folder_id = models::folders::PublicId::try_from(payload.folder_id)?;

        let public_key = SenderPublicKey::try_from(payload.public_key)
            .map_err(|_| Status::invalid_argument("invalid public key"))?;

        let result = self
            .share_service
            .initiate_session(folder_id, public_key)
            .await
            .split_business()?
            .map_err(|e| match e {
                share::service::InitiateSessionError::FolderNotFound => {
                    Status::not_found("folder not found")
                }
            })?;

        let (session_id, stream) = result;
        let sid_str = session_id.to_string();

        let proto_stream = stream.map(move |event| {
            Ok(ProtoShareEvent {
                session_id: sid_str.clone(),
                event: Some((&event).into()),
            })
        });

        Ok(Response::new(proto_stream))
    }

    type JoinStream = impl Stream<Item = Result<ProtoShareEvent, Status>> + Send + 'static;

    async fn join(
        &self,
        request: Request<JoinRequest>,
    ) -> Result<Response<Self::JoinStream>, Status> {
        let payload = request.into_inner();
        let code = Code::from(payload.code);
        let public_key = ReceiverPublicKey::try_from(payload.public_key)
            .map_err(|_| Status::invalid_argument("invalid public key"))?;

        let result = self
            .share_service
            .join_session(code, public_key)
            .await
            .split_business()?
            .map_err(|e| match e {
                share::service::JoinSessionError::InvalidCode => {
                    Status::invalid_argument("invalid code")
                }
                share::service::JoinSessionError::SessionNotFound => {
                    Status::not_found("session not found")
                }
            })?;

        let (session_id, stream) = result;
        let sid_str = session_id.to_string();

        let proto_stream = stream.map(move |event| {
            Ok(ProtoShareEvent {
                session_id: sid_str.clone(),
                event: Some((&event).into()),
            })
        });

        Ok(Response::new(proto_stream))
    }

    async fn send_key(
        &self,
        request: Request<SendKeyRequest>,
    ) -> Result<Response<SendKeyResponse>, Status> {
        let payload = request.into_inner();
        let session_id = SessionId::from_str(&payload.session_id)
            .map_err(|_| Status::invalid_argument("invalid session id"))?;
        let folder_key = payload.encrypted_folder_key;

        self.share_service
            .send_key(session_id, folder_key)
            .await
            .split_business()?
            .map_err(|e| match e {
                SendKeyError::SessionNotFound => Status::not_found("session not found"),
                SendKeyError::Unauthorized => Status::permission_denied("unauthorized"),
            })?;

        Ok(Response::new(SendKeyResponse { success: true }))
    }
}
