use crate::schema::api::folder::v1::files_service_server::FilesService;
use crate::schema::api::folder::v1::{
    Blob, DeleteRequest, DownloadRequest, ListFilesRequest, ListFilesResponse, UploadFileRequest,
    UploadFileResponse, upload_file_request,
};
use crate::schema::{BoolExt, IntoInternal, ServiceErrorExt, ServiceResultExt};
use async_trait::async_trait;
use auth::service::TokenService;
use derive_new::new;
use domain::entity;
use download::service::DownloadService;
use futures::Stream;
use futures_util::TryStreamExt;
use pbjson_types::Empty;
use tonic::codegen::tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};
use upload::service::{UploadFileError, UploadService};

#[derive(Debug, Clone, new)]
pub struct BasicGrpcFilesService<FilesS, FoldersS, DS, US, TS> {
    files_service: FilesS,
    folders_service: FoldersS,
    download_service: DS,
    upload_service: US,
    token_service: TS,
}

#[async_trait]
impl<FilesS, FoldersS, DS, US, TS> FilesService
    for BasicGrpcFilesService<FilesS, FoldersS, DS, US, TS>
where
    FilesS: files::service::FilesService,
    FoldersS: folders::service::FoldersService,
    DS: DownloadService,
    US: UploadService,
    TS: TokenService,
{
    async fn list_files(
        &self,
        request: Request<ListFilesRequest>,
    ) -> Result<Response<ListFilesResponse>, Status> {
        let payload = request.into_inner();
        let folder: entity::folders::PublicId = payload.folder.try_into()?;

        let folder = self
            .folders_service
            .find_folder_by_public_id(folder)
            .await
            .ok_or_internal()?
            .ok_or_not_found(None::<&str>)?;

        let files = self
            .files_service
            .list_folder_files(folder.id)
            .await
            .ok_or_internal()?;

        Ok(Response::new(ListFilesResponse {
            files: files.into_iter().map(Into::into).collect(),
        }))
    }

    async fn upload_file(
        &self,
        request: Request<Streaming<UploadFileRequest>>,
    ) -> Result<Response<UploadFileResponse>, Status> {
        let mut stream = request.into_inner();

        let Some(Ok(UploadFileRequest {
            data: Some(upload_file_request::Data::Initiate(initiate)),
        })) = stream.next().await
        else {
            return Err(Status::invalid_argument("invalid initial request"));
        };

        let public_id = entity::folders::PublicId::try_from(initiate.folder.folder_id)?;
        let result: Result<_, _> = self
            .upload_service
            .upload_file_by_public_folder_id(
                public_id,
                initiate.folder.token.value,
                initiate.metadata.encrypted_path,
                initiate.metadata.encrypted_mime,
                initiate.metadata.encrypted_hash,
                stream.map(|item| match item {
                    Ok(UploadFileRequest {
                        data: Some(upload_file_request::Data::ChunkData(bytes)),
                    }) => Ok(bytes),
                    _ => Err(Status::aborted("aborted")),
                }),
            )
            .await
            .ok_or_internal()?;

        let file = result.map_err(|e| match e {
            UploadFileError::FileTooLarge => Status::invalid_argument("file too large"),
            UploadFileError::FolderNotFound => Status::not_found("folder not found"),
            UploadFileError::NoPermissions => Status::unauthenticated("no permissions"),
            UploadFileError::FolderIsFull => Status::aborted("folder is full"),
            UploadFileError::Stream(_) => Status::cancelled("stream cancelled"),
        })?;

        Ok(Response::new(UploadFileResponse {
            id: file.public_id.into(),
        }))
    }

    type DownloadStream = impl Stream<Item = Result<Blob, Status>>;

    async fn download(
        &self,
        request: Request<DownloadRequest>,
    ) -> Result<Response<Self::DownloadStream>, Status> {
        let inner = request.into_inner();
        let folder_id = entity::folders::PublicId::try_from(inner.folder)?;
        let file_id = entity::files::PublicId::try_from(inner.file)?;

        let stream = self
            .download_service
            .download_file_stream_by_public_ids(folder_id, file_id)
            .await
            .ok_or_internal()?
            .ok_or_not_found(None::<&str>)?;

        Ok(Response::new(
            stream
                .map_err(IntoInternal::into_internal)
                .map_ok(|part| Blob { part }),
        ))
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<Empty>, Status> {
        let payload = request.into_inner();
        let folder: entity::folders::PublicId = payload.folder.folder_id.try_into()?;
        let file: entity::files::PublicId = payload.file_id.try_into()?;
        let token = payload.folder.token.value;

        self.token_service
            .is_token_valid_for_folder(&folder, token)
            .await
            .ok_or_internal()?
            .ok_or_unauthenticated()?;

        let folder = self
            .folders_service
            .find_folder_by_public_id(folder)
            .await
            .ok_or_internal()?
            .ok_or_not_found("folder not found")?;

        self
            .files_service
            .delete_file_from_folder_by_public_id(folder.id, file)
            .await
            .ok_or_internal()?
            .ok_or_not_found("file not found")?;
        
        Ok(Response::new(Empty {}))
    }
}
