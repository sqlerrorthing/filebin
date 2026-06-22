use crate::schema::api::folder::v1::folder_service_server::FolderService;
use crate::schema::api::folder::v1::{
    CreateFolderRequest, OwnedFolder, UploadFileRequest, UploadFileResponse, upload_file_request,
};
use crate::schema::{ServiceErrorExt, ServiceResultExt};
use crate::v1::dto::prost_duration_to_std_duration;
use async_trait::async_trait;
use auth::service::TokenService;
use derive_new::new;
use domain::entity;
use files::service::UploadFileError;
use tonic::codegen::tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

#[derive(new)]
pub struct BasicGrpcFolderService<FilesS, FolderS, TS> {
    files_service: FilesS,
    folders_service: FolderS,
    token_service: TS,
}

#[async_trait]
impl<FilesS, FolderS, TS> FolderService for BasicGrpcFolderService<FilesS, FolderS, TS>
where
    FilesS: files::service::FilesService,
    FolderS: folders::service::FoldersService,
    TS: TokenService,
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
            token: token.into(),
        }))
    }

    async fn upload_file(
        &self,
        request: Request<Streaming<UploadFileRequest>>,
    ) -> Result<Response<UploadFileResponse>, Status> {
        let mut stream = request.into_inner();

        let Some(Ok(UploadFileRequest {
            data: Some(upload_file_request::Data::Initiate(initiate)),
        })) = dbg!(stream.next().await)
        else {
            return Err(Status::invalid_argument("invalid initial request"));
        };

        let public_id = entity::folders::PublicId::try_from(initiate.id)?;

        if !self
            .token_service
            .is_token_valid_for_folder(&public_id, initiate.token.value)
            .await
            .ok_or_internal()?
        {
            return Err(Status::unauthenticated("invalid token"));
        }

        let folder = self
            .folders_service
            .find_folder_by_public_id(public_id)
            .await
            .ok_or_internal()?
            .ok_or_not_found("folder not found")?;

        let chunks = stream.map(|item| match item {
            Ok(UploadFileRequest {
                data: Some(upload_file_request::Data::ChunkData(bytes)),
            }) => Ok(bytes),
            _ => Err(Status::aborted("aborted")),
        });

        let upload: Result<_, _> = self
            .files_service
            .upload_file(
                folder.id,
                initiate.metadata.encrypted_path,
                initiate.metadata.encrypted_mime,
                initiate.metadata.encrypted_hash,
                chunks,
            )
            .await
            .ok_or_internal()?;

        let file = upload.map_err(|err| match err {
            UploadFileError::FileTooLarge => Status::aborted("file too large"),
            UploadFileError::Stream(_) => Status::aborted("aborted due to stream interrupt"),
        })?;

        Ok(Response::new(UploadFileResponse {
            id: file.public_id.into(),
        }))
    }
}
