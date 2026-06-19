use async_trait::async_trait;
use derive_new::new;
use tonic::{Request, Response, Status};
use crate::schema::api::folder::v1::files_service_server::FilesService;
use crate::schema::api::folder::v1::{ListFilesRequest, ListFilesResponse};

#[derive(Debug, Clone, new)]
pub struct BasicGrpcFilesService<FS> {
    files_service: FS
}

#[async_trait]
impl<FS> FilesService for BasicGrpcFilesService<FS>
where
    FS: files::service::FilesService
{
    async fn list_files(&self, request: Request<ListFilesRequest>) -> Result<Response<ListFilesResponse>, Status> {
        todo!()
    }
}