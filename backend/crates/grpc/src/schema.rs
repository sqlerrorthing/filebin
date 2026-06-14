use std::error::Error;
use tonic::Status;
use tracing::error;

pub mod api {
    pub mod folder {
        pub mod v1 {
            tonic::include_proto!("folder.v1");
        }
    }
}

pub trait ServiceResultExt<T> {
    fn ok_or_internal(self) -> Result<T, Status>;
}

impl<T, E: Error> ServiceResultExt<T> for Result<T, E> {
    #[inline(always)]
    fn ok_or_internal(self) -> Result<T, Status> {
        self.map_err(|e| {
            error!("Server internal error: {e}");

            Status::internal(if cfg!(debug_assertions) {
                format!("server error: {e}")
            } else {
                "server error".to_string()
            })
        })
    }
}
