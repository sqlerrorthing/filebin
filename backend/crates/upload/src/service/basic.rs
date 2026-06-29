use crate::service::{UploadFileError, UploadService};
use auth::service::TokenService;
use bytes::Bytes;
use derive_builder::Builder;
use derive_new::new;
use domain::models;
use files::service::FilesService;
use folders::service::FoldersService;
use futures::{Stream, StreamExt, TryStreamExt};
use service::error::{OptionExt, ResultExt, ServiceError};
use thiserror::Error;
use updates::service::UpdatesService;
use crate::limited_stream::{LimitStreamError, LimitedStream};

#[derive(Builder, Debug, Clone)]
pub struct Limits {
    max_filesize: u64,
    max_files_per_folder: u32,
}

#[derive(Debug, Clone, new)]
pub struct BasicUploadService<FilesS, FoldersS, TS, US> {
    files_service: FilesS,
    folders_service: FoldersS,
    token_service: TS,
    updates_service: US,
    limits: Limits,
}

#[derive(Debug, Error)]
pub enum Error<FilesS: FilesService, FoldersS: FoldersService, TS: TokenService> {
    #[error("files service error: {0}")]
    Files(#[source] FilesS::Error),
    #[error("folders service error: {0}")]
    Folders(#[source] FoldersS::Error),
    #[error("token service error: {0}")]
    Token(#[source] TS::Error),
}

impl<FilesS, FoldersS, TS, US> UploadService for BasicUploadService<FilesS, FoldersS, TS, US>
where
    FilesS: FilesService,
    FoldersS: FoldersService,
    TS: TokenService,
    US: UpdatesService
{
    type Error = Error<FilesS, FoldersS, TS>;

    async fn upload_file_by_public_folder_id<E: std::error::Error>(
        &self,
        public_id: models::folders::PublicId,
        token: String,
        encrypted_path: String,
        encrypted_mime_type: String,
        encrypted_file_hash: String,
        chunks: impl Stream<Item = Result<Bytes, E>> + Send + 'static,
    ) -> Result<models::files::Model, ServiceError<UploadFileError<E>, Self::Error>>
    where
        E: Send + 'static,
    {
        self
            .token_service
            .is_token_valid_for_folder(&public_id, token)
            .await
            .map_err(Error::Token)?
            .ok_or_business(UploadFileError::NoPermissions)?;
        
        let folder = self.folders_service.find_folder_by_public_id(public_id)
            .await
            .map_err(Error::Folders)?
            .ok_or_business(UploadFileError::FolderNotFound)?;

        self.files_service.files_count(folder.id).await.map_err(Error::Files)?
            .lt(&(self.limits.max_files_per_folder as u64))
            .ok_or_business(UploadFileError::FolderIsFull)?;

        let res = self.files_service.upload_file(
            folder.id,
            encrypted_path,
            encrypted_mime_type,
            encrypted_file_hash,
            LimitedStream::new(chunks, self.limits.max_filesize)
                .map_err(|err| {
                    match err {
                        LimitStreamError::LimitExceeds => UploadFileError::FileTooLarge,
                        LimitStreamError::Stream(s) => UploadFileError::Stream(s)
                    }
                })
        ).await.map_internal(Error::Files)?;
        
        self.updates_service.fire_file_uploaded(res.clone());
        Ok(res)
    }
}
