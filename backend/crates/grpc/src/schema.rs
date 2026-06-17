use std::error::Error;
use tonic::Status;
use tracing::error;
use domain::into_string::IntoOptionalString;

pub mod api {
    pub mod folder {
        pub mod v1 {
            tonic::include_proto!("folder.v1");
        }
    }

    pub mod google {
        pub mod r#type {
            tonic::include_proto!("google.r#type");
        }
    }
}

pub trait ServiceResultExt<T> {
    fn ok_or_internal(self) -> Result<T, Status>;

    fn ok_or_invalid_argument(self, msg: impl IntoOptionalString) -> Result<T, Status>;
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

    fn ok_or_invalid_argument(self, msg: impl IntoOptionalString) -> Result<T, Status> {
        self.map_err(|e| {
            let mut base = "invalid argument".to_string();

            if let Some(msg) = msg.as_ref() {
                base += &format!(": {msg}");
            }

            if cfg!(debug_assertions) {
                base += &format!("; {e}")
            }

            Status::invalid_argument(base)
        })
    }
}
