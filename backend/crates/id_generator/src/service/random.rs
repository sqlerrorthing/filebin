use rand::{fill, rng, RngExt};
use rand::distr::Alphanumeric;
use rand::prelude::IteratorRandom;
use tinystr::{TinyAsciiStr, TinyStr16, TinyStr8};
use domain::entity::files;
use domain::entity::folders;
use crate::service::IdGeneratorService;

#[derive(Debug, Copy, Clone)]
pub struct RandomIdGeneratorService;

impl IdGeneratorService for RandomIdGeneratorService {
    fn next_public_folder_id(&self) -> folders::PublicId {
        folders::PublicId::new(fill_tinystr())
    }

    fn next_public_file_id(&self) -> files::PublicId {
        files::PublicId::new(fill_tinystr())
    }
}

fn fill_tinystr<const N: usize>() -> TinyAsciiStr<N> {
    let mut buf = [0u8; N];
    rng().sample_iter(Alphanumeric).sample_fill(&mut rng(), &mut buf);
    unsafe { TinyAsciiStr::<N>::try_from_raw(buf).unwrap_unchecked() }
}
