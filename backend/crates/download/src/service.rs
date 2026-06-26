pub mod basic;

use bytes::Bytes;
use domain::entity::{files, folders};
use futures_core::Stream;
use service::service;

#[service]
pub trait DownloadService {
    type Error;

    type DownloadFileByPublicIdsStream: Stream<Item = Result<Bytes, Self::Error>>;

    #[result]
    async fn download_file_stream_by_public_ids(
        &self,
        folder_id: folders::PublicId,
        file_id: files::PublicId,
    ) -> Option<Self::DownloadFileByPublicIdsStream>;
}
