use crate::repository::FilesRepository;
use crate::service::{FilesService, UploadFileError};
use crate::storage::{FILES_PREFIX, FilesStorage};
use bytes::Bytes;
use derive_new::new;
use domain::entity::{files, folders};
use id_generator::service::IdGeneratorService;
use sea_orm::Set;
use service::business;
use service::error::ServiceError;
use thiserror::Error;
use tokio::spawn;
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

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
    Files(#[source] FS::Error),
    #[error("files repository error: {0}")]
    Repository(#[source] FR::Error),
}

impl<FS, FR, IGS> FilesService for BasicFilesService<FS, FR, IGS>
where
    FS: FilesStorage,
    FR: FilesRepository,
    IGS: IdGeneratorService,
    Self: Clone,
{
    type Error = Error<FS, FR>;

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
            .map_err(Error::Files)
    }

    async fn list_folder_files(&self, folder_id: folders::Id) -> Result<Vec<files::Model>, Self::Error> {
        self.files_repository
            .list_folder_files(folder_id)
            .await
            .map_err(Error::Repository)
    }

    fn upload_file(
        &self,
        folder_id: folders::Id,
        encrypted_path: String,
        encrypted_mime_type: String,
        encrypted_file_hash: String,
        mut chunks: mpsc::Receiver<Bytes>,
        cancellation: CancellationToken,
    ) -> oneshot::Receiver<Result<files::Model, ServiceError<UploadFileError, Self::Error>>> {
        let (mut tx, rx) = oneshot::channel();

        let this = self.clone();

        spawn(async move {
            let result = async {
                let storage_path = this.id_generator_service.next_file_storage_path();

                let handle = this
                    .files_storage
                    .create_multipart_upload(format!("{FILES_PREFIX}/{storage_path}"))
                    .await
                    .map_err(Error::Files)?;

                let mut total_bytes_received = 0_u64;

                loop {
                    let chunk = tokio::select! {
                        _ = tx.closed() => {
                            return Err(business!(UploadFileError::Cancelled))
                        }
                        _ = cancellation.cancelled() => {
                            return Err(business!(UploadFileError::Cancelled))
                        }
                        maybe_chunk = chunks.recv() => {
                            match maybe_chunk {
                                Some(c) => c,
                                None => break
                            }
                        }
                    };

                    let chunk_len = chunk.len() as u64;
                    if total_bytes_received + chunk_len > this.max_filesize {
                        return Err(business!(UploadFileError::FileTooLarge));
                    }

                    total_bytes_received += chunk_len;

                    this.files_storage
                        .upload_part(&handle, chunk)
                        .await
                        .map_err(Error::Files)?;
                }

                if cancellation.is_cancelled() || tx.is_closed() {
                    return Err(business!(UploadFileError::Cancelled));
                }

                this.files_storage
                    .complete_multipart_upload(handle)
                    .await
                    .map_err(Error::Files)?;

                let model = this
                    .files_repository
                    .insert(files::ActiveModel {
                        public_id: Set(this.id_generator_service.next_public_file_id()),
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
            .await;

            let _ = tx.send(result);
        });

        rx
    }
}
