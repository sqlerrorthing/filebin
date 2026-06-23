use crate::schema::api::folder::v1::files_service_server::FilesService;
use crate::schema::api::folder::v1::{upload_file_request, Blob, DownloadRequest, ListFilesRequest, ListFilesResponse, UploadFileRequest, UploadFileResponse};
use crate::schema::{IntoInternal, ServiceErrorExt, ServiceResultExt};
use async_trait::async_trait;
use derive_new::new;
use domain::entity;
use download::service::DownloadService;
use futures::Stream;
use futures_util::TryStreamExt;
use tonic::{Request, Response, Status, Streaming};
use tonic::codegen::tokio_stream::StreamExt;
use auth::service::TokenService;
use files::service::UploadFileError;

#[derive(Debug, Clone, new)]
pub struct BasicGrpcFilesService<FilesS, FoldersS, DS, TS> {
    files_service: FilesS,
    folders_service: FoldersS,
    download_service: DS,
    token_service: TS,
}

#[async_trait]
impl<FilesS, FoldersS, DS, TS> FilesService for BasicGrpcFilesService<FilesS, FoldersS, DS, TS>
where
    FilesS: files::service::FilesService,
    FoldersS: folders::service::FoldersService,
    DS: DownloadService,
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
}
