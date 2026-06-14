use async_trait::async_trait;
use derive_new::new;
use tonic::{Request, Response, Status};
use crate::schema::api::folder::v1::folder_service_server::FolderService;
use crate::schema::api::folder::v1::OwnedFolder;

#[derive(Debug, Clone, new)]
pub struct BasicGrpcFolderService {

}

#[async_trait]
impl FolderService for BasicGrpcFolderService {
    async fn create_folder(&self, request: Request<()>) -> Result<Response<OwnedFolder>, Status> {
        todo!()
    }
}