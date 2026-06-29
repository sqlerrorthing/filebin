pub mod files;
pub mod folders;
pub mod encrypted_blobs;
pub mod encrypted_vault;

#[derive(Copy, Clone,  Default)]
pub enum ActiveValue<T> {
    #[default]
    NotSet,
    Set(T),
    Unchanged(T),
}
