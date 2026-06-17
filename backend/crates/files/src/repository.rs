pub mod db;

use domain::entity::files;
use service::service;

#[service]
pub trait FilesRepository: 'static {
    type Error;

    #[result]
    async fn insert(&self, folder: files::ActiveModel) -> files::Model;
    
    #[result]
    async fn update(&self, folder: files::ActiveModel) -> files::Model;
}
