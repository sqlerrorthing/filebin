use crate::schema::ServiceResultExt;
use crate::schema::api::folder::v1::folder_service_server::FolderService;
use crate::schema::api::folder::v1::{CreateFolderRequest, OwnedFolder};
use crate::v1::dto::{prost_duration_to_datetime_duration, prost_duration_to_std_duration};
use async_trait::async_trait;
use derive_new::new;
use tonic::{Request, Response, Status};

#[derive(Debug, Clone, new)]
pub struct BasicGrpcFolderService<FS, TS> {
    folders_service: FS,
    token_service: TS,
}

#[async_trait]
impl<FS: folders::service::FoldersService, TS: auth::service::TokenService> FolderService
    for BasicGrpcFolderService<FS, TS>
{
    async fn create_folder(
        &self,
        request: Request<CreateFolderRequest>,
    ) -> Result<Response<OwnedFolder>, Status> {
        let payload = request.get_ref();

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
            token: token.as_str().into(),
        }))
    }
}
