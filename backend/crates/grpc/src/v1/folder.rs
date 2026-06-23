use async_trait::async_trait;
use derive_new::new;
use pbjson_types::Empty;
use crate::schema::api::folder::v1::folder_service_server::FolderService;
use crate::schema::api::folder::v1::{CreateFolderRequest, DeleteFolderRequest, DeleteFolderResponse, OwnedFolder, UploadFileRequest, UploadFileResponse, upload_file_request, LimitsResponse};
use crate::schema::{ServiceErrorExt, ServiceResultExt};
use crate::v1::dto::prost_duration_to_std_duration;
use auth::service::TokenService;
use tonic::{Request, Response, Status};
use domain::entity;
use crate::config::CONFIG;

#[derive(new)]
pub struct BasicGrpcFolderService<FS, TS> {
    folders_service: FS,
    token_service: TS,
}

#[async_trait]
impl<FS, TS> FolderService for BasicGrpcFolderService<FS, TS>
where
    FS: folders::service::FoldersService,
    TS: TokenService,
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
    ) -> Result<Response<DeleteFolderResponse>, Status> {
        let payload = request.into_inner();
        let id: entity::folders::PublicId = payload.id.try_into()?;
        let token = payload.token.value;

        if !self.token_service.is_token_valid_for_folder(&id, token).await.ok_or_internal()? {
            return Err(Status::unauthenticated("invalid token"))
        }

        let folder = self.folders_service.find_folder_by_public_id(id)
            .await
            .ok_or_internal()?
            .ok_or_not_found("folder not found")?;

        if self.folders_service.delete_folder(folder.id).await.ok_or_internal()? {
            Ok(Response::new(DeleteFolderResponse {}))
        } else {
            Err(Status::not_found("folder not found"))
        }
    }

    async fn limits(&self, _: Request<Empty>) -> Result<Response<LimitsResponse>, Status> {
        Ok(Response::new(
            LimitsResponse {
                max_files_per_folder: CONFIG.limits.max_files_per_folder,
                max_file_size: CONFIG.limits.max_filesize.as_u64()
            }
        ))
    }
}
