use crate::repository::FilesRepository;
use crate::service::{FilesService, UploadFileError};
use crate::storage::{FILES_PREFIX, FilesStorage};
use bytes::Bytes;
use derive_new::new;
use domain::entity::{files, folders};
use futures_core::Stream;
use futures_util::{StreamExt, TryStreamExt};
use id_generator::service::IdGeneratorService;
use sea_orm::Set;
use service::business;
use service::error::ServiceError;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Clone, Debug, new)]
pub struct BasicFilesService<FS, FR, IGS> {
    files_storage: FS,
    files_repository: FR,
    id_generator_service: IGS,
    max_filesize: u64,
}

#[derive(Debug, Error)]
pub enum Error<FS: FilesStorage, FR: FilesRepository> {
    #[error("files storage error: {0}")]
    Storage(#[source] FS::Error),
    #[error("files repository error: {0}")]
    Repository(#[source] FR::Error),
}

impl<FS, FR, IGS> FilesService for BasicFilesService<FS, FR, IGS>
where
    FS: FilesStorage,
    FR: FilesRepository,
    IGS: IdGeneratorService,
{
    type Error = Error<FS, FR>;
    type GetFileStream = impl Stream<Item = Result<Bytes, Self::Error>> + Debug;

    fn min_upload_chunk_size(&self) -> i64 {
        5 * 1024 * 1024
    }

    async fn delete_files_from_folder(&self, folder_id: folders::Id) -> Result<(), Self::Error> {
        let files = self
            .files_repository
            .delete_files_from_folder(folder_id)
            .await
            .map_err(Error::Repository)?;

        if files.is_empty() {
            return Ok(());
        }

        self.files_storage
            .bulk_delete(files.into_iter().map(|f| f.storage_path).collect())
            .await
            .map_err(Error::Storage)
    }

    async fn list_folder_files(
        &self,
        folder_id: folders::Id,
    ) -> Result<Vec<files::Model>, Self::Error> {
        self.files_repository
            .list_folder_files(folder_id)
            .await
            .map_err(Error::Repository)
    }

    async fn find_file_by_public_id_in_folder_by_id(
        &self,
        folder_id: folders::Id,
        public_id: files::PublicId,
    ) -> Result<Option<files::Model>, Self::Error> {
        Ok(self
            .files_repository
            .find_file_by_public_id(public_id)
            .await
            .map_err(Error::Repository)?
            .take_if(|f| f.folder_id == folder_id))
    }

    async fn upload_file<E>(
        &self,
        folder_id: folders::Id,
        encrypted_path: String,
        encrypted_mime_type: String,
        encrypted_file_hash: String,
        chunks: impl Stream<Item = Result<Bytes, E>> + Send + 'static,
    ) -> Result<files::Model, ServiceError<UploadFileError<E>, Self::Error>>
    where
        E: Send + 'static,
    {
        let storage_path = self.id_generator_service.next_file_storage_path();
        tokio::pin!(chunks);

        let handle = self
            .files_storage
            .create_multipart_upload(storage_path)
            .await
            .map_err(Error::Storage)?;

        let mut total_bytes_received = 0_u64;

        loop {
            let chunk = match chunks.next().await {
                Some(Ok(c)) => c,
                Some(Err(e)) => return Err(business!(UploadFileError::Stream(e))),
                None => break,
            };

            let chunk_len = chunk.len() as u64;
            if total_bytes_received + chunk_len > self.max_filesize {
                return Err(business!(UploadFileError::FileTooLarge));
            }

            total_bytes_received += chunk_len;

            self.files_storage
                .upload_part(&handle, chunk)
                .await
                .map_err(Error::Storage)?;
        }

        let storage_path = self
            .files_storage
            .complete_multipart_upload(handle)
            .await
            .map_err(Error::Storage)?;

        let model = self
            .files_repository
            .insert(files::ActiveModel {
                public_id: Set(self.id_generator_service.next_public_file_id()),
                folder_id: Set(folder_id),
                storage_path: Set(storage_path),
                encrypted_path: Set(encrypted_path),
                encrypted_mime_type: Set(encrypted_mime_type),
                encrypted_file_hash: Set(encrypted_file_hash),
                file_size: Set(total_bytes_received as _),
                ..Default::default()
            })
            .await
            .map_err(Error::Repository)?;

        Ok(model)
    }

    async fn get_file_by_storage_path(
        &self,
        storage_path: files::StoragePath,
    ) -> Result<Option<Self::GetFileStream>, Self::Error> {
        Ok(self
            .files_storage
            .get_file(storage_path)
            .await
            .map_err(Error::Storage)?
            .map(|stream| stream.map_err(Error::Storage)))
    }
}
