use crate::schema::api::folder::v1::files_service_server::FilesService;
use crate::schema::api::folder::v1::{Blob, DownloadRequest, ListFilesRequest, ListFilesResponse};
use crate::schema::{IntoInternal, ServiceErrorExt, ServiceResultExt};
use async_trait::async_trait;
use derive_new::new;
use domain::entity;
use download::service::DownloadService;
use futures::Stream;
use futures_util::TryStreamExt;
use tonic::{Request, Response, Status};

#[derive(Debug, Clone, new)]
pub struct BasicGrpcFilesService<FilesS, FoldersS, DS> {
    files_service: FilesS,
    folders_service: FoldersS,
    download_service: DS,
}

#[async_trait]
impl<FilesS, FoldersS, DS> FilesService for BasicGrpcFilesService<FilesS, FoldersS, DS>
where
    FilesS: files::service::FilesService,
    FoldersS: folders::service::FoldersService,
    DS: DownloadService,
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
