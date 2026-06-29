use futures_util::TryStreamExt;
use crate::service::DownloadService;
use bytes::Bytes;
use derive_new::new;
use domain::persistance;
use files::service::FilesService;
use folders::service::FoldersService;
use futures_core::Stream;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Clone, new)]
pub struct BasicDownloadService<FilesS, FoldersS> {
    files_service: FilesS,
    folders_service: FoldersS,
}

#[derive(Debug, Error)]
pub enum Error<FilesS, FoldersS>
where
    FilesS: FilesService,
    FoldersS: FoldersService,
{
    #[error("files error: {0}")]
    Files(#[source] FilesS::Error),
    #[error("folders error: {0}")]
    Folders(#[source] FoldersS::Error),
}

impl<FilesS, FoldersS> DownloadService for BasicDownloadService<FilesS, FoldersS>
where
    FilesS: FilesService,
    FoldersS: FoldersService,
{
    type Error = Error<FilesS, FoldersS>;
    type DownloadFileByPublicIdsStream =
        impl Stream<Item = Result<Bytes, Self::Error>> + Debug;

    async fn download_file_stream_by_public_ids(
        &self,
        folder_id: persistance::folders::PublicId,
        file_id: persistance::files::PublicId,
    ) -> Result<Option<Self::DownloadFileByPublicIdsStream>, Self::Error> {
        let Some(folder) = self
            .folders_service
            .find_folder_by_public_id(folder_id)
            .await
            .map_err(Error::Folders)?
        else {
            return Ok(None);
        };

        let Some(file) = self
            .files_service
            .find_file_by_public_id_in_folder_by_id(folder.id, file_id)
            .await
            .map_err(Error::Files)?
        else {
            return Ok(None);
        };

        Ok(self
            .files_service
            .get_file_by_storage_path(file.storage_path)
            .await
            .map_err(Error::Files)?
            .map(|s| s.map_err(Error::Files))
        )
    }
}
