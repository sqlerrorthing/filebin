use async_trait::async_trait;
use derive_new::new;
use tonic::{Request, Response, Status};
use domain::entity;
use crate::schema::api::folder::v1::{ListFilesRequest, ListFilesResponse};
use crate::schema::api::folder::v1::files_service_server::FilesService;
use crate::schema::{ServiceErrorExt, ServiceResultExt};

#[derive(Debug, Clone, new)]
pub struct BasicGrpcFilesService<FilesS, FoldersS> {
    files_service: FilesS,
    folders_service: FoldersS
}

#[async_trait]
impl<FilesS, FoldersS> FilesService for BasicGrpcFilesService<FilesS, FoldersS>
where
    FilesS: files::service::FilesService,
    FoldersS: folders::service::FoldersService,
{
    async fn list_files(&self, request: Request<ListFilesRequest>) -> Result<Response<ListFilesResponse>, Status> {
        let payload = request.into_inner();
        let folder: entity::folders::PublicId = payload.folder.try_into()?;

        let folder = self.folders_service.find_folder_by_public_id(folder).await
            .ok_or_internal()?
            .ok_or_not_found(None::<&str>)?;

        let files = self.files_service.list_folder_files(folder.id).await
            .ok_or_internal()?;

        Ok(Response::new(
            ListFilesResponse {
                files: files.into_iter().map(Into::into).collect(),
            }
        ))
    }
}
