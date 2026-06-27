use crate::config::CONFIG;
use crate::schema::api::folder::v1::folder_service_server::FolderService;
use crate::schema::api::folder::v1::{
    CreateFolderRequest, DeleteFolderRequest, FolderDeleted, FolderNameChanged, FolderUpdate,
    LimitsResponse, NewFile, OwnedFolder, RenameRequest, UpdatesRequest, UploadFileRequest,
    UploadFileResponse, folder_update, upload_file_request,
};
use crate::schema::{ServiceErrorExt, ServiceResultExt};
use crate::v1::dto::prost_duration_to_std_duration;
use async_trait::async_trait;
use auth::service::TokenService;
use derive_new::new;
use domain::entity;
use futures::Stream;
use pbjson_types::Empty;
use std::ops::Deref;
use std::sync::Arc;
use tonic::codegen::tokio_stream::StreamExt;
use tonic::{Request, Response, Status};
use updates::service::UpdatesService;

#[derive(new)]
pub struct BasicGrpcFolderService<FS, TS, US> {
    folders_service: FS,
    token_service: TS,
    updates_service: US,
}

#[async_trait]
impl<FS, TS, US> FolderService for BasicGrpcFolderService<FS, TS, US>
where
    FS: folders::service::FoldersService,
    TS: TokenService,
    US: UpdatesService,
{
    async fn create_folder(
        &self,
        request: Request<CreateFolderRequest>,
    ) -> Result<Response<OwnedFolder>, Status> {
        let payload = request.into_inner();

        let folder = self
            .folders_service
            .create_folder(
                payload.encrypted_name.clone(),
                payload
                    .expires
                    .map(prost_duration_to_std_duration)
                    .transpose()
                    .ok_or_invalid_argument("invalid expiration")?,
            )
            .await
            .ok_or_internal()?;

        let token = self
            .token_service
            .generate_token_for_folder_public_id(&folder.public_id)
            .await
            .ok_or_internal()?;

        Ok(Response::new(OwnedFolder {
            folder: folder.into(),
            token: token.into(),
        }))
    }

    async fn delete_folder(
        &self,
        request: Request<DeleteFolderRequest>,
    ) -> Result<Response<Empty>, Status> {
        let payload = request.into_inner();
        let id: entity::folders::PublicId = payload.owned_folder.folder_id.try_into()?;
        let token = payload.owned_folder.token.value;

        if !self
            .token_service
            .is_token_valid_for_folder(&id, token)
            .await
            .ok_or_internal()?
        {
            return Err(Status::unauthenticated("invalid token"));
        }

        let folder = self
            .folders_service
            .find_folder_by_public_id(id)
            .await
            .ok_or_internal()?
            .ok_or_not_found("folder not found")?;

        if self
            .folders_service
            .delete_folder(folder.id)
            .await
            .ok_or_internal()?
        {
            Ok(Response::new(Empty {}))
        } else {
            Err(Status::not_found("folder not found"))
        }
    }

    async fn limits(&self, _: Request<Empty>) -> Result<Response<LimitsResponse>, Status> {
        Ok(Response::new(LimitsResponse {
            max_files_per_folder: CONFIG.limits.max_files_per_folder,
            max_file_size: CONFIG.limits.max_filesize.as_u64(),
        }))
    }

    async fn rename(&self, request: Request<RenameRequest>) -> Result<Response<Empty>, Status> {
        let payload = request.into_inner();
        let id: entity::folders::PublicId = payload.owned_folder.folder_id.try_into()?;
        let token = payload.owned_folder.token.value;

        if !self
            .token_service
            .is_token_valid_for_folder(&id, token)
            .await
            .ok_or_internal()?
        {
            return Err(Status::unauthenticated("invalid token"));
        }

        let folder = self
            .folders_service
            .find_folder_by_public_id(id)
            .await
            .ok_or_internal()?
            .ok_or_not_found("folder not found")?;

        self.folders_service
            .rename_folder(folder.id, payload.encrypted_name)
            .await
            .ok_or_internal()?
            .ok_or_not_found("folder not found")?;
        
        Ok(Response::new(Empty {}))
    }

    type UpdatesStream = impl Stream<Item = Result<FolderUpdate, Status>>;

    async fn updates(
        &self,
        request: Request<UpdatesRequest>,
    ) -> Result<Response<Self::UpdatesStream>, Status> {
        let folder_id: entity::folders::PublicId = request.into_inner().id.try_into()?;
        let folder = self
            .folders_service
            .find_folder_by_public_id(folder_id)
            .await
            .ok_or_internal()?
            .ok_or_not_found("folder not found")?;

        let stream = self
            .updates_service
            .subscribe_folder(folder.id)
            .map(|update| {
                Ok(FolderUpdate {
                    update: Some((&update.kind).into()),
                })
            });

        Ok(Response::new(stream))
    }
}
