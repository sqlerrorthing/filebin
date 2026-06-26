pub mod random;

use domain::entity::{files, folders};
use service::service;

#[service]
pub trait IdGeneratorService {
    fn next_public_folder_id(&self) -> folders::PublicId;
    
    fn next_public_file_id(&self) -> files::PublicId;
    
    fn next_file_storage_path(&self) -> files::StoragePath;
}
