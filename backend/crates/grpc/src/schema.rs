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

pub trait ServiceErrorExt<T> {
    fn ok_or_invalid_argument(self, msg: impl IntoOptionalString) -> Result<T, Status>;

    fn ok_or_not_found(self, msg: impl IntoOptionalString) -> Result<T, Status>;
}

impl<T> ServiceErrorExt<T> for Option<T> {
    fn ok_or_invalid_argument(self, msg: impl IntoOptionalString) -> Result<T, Status> {
        self.ok_or_else(|| {
            let mut base = "invalid argument".to_string();

            if let Some(msg) = msg.as_ref() {
                base += &format!(": {msg}");
            }

            Status::invalid_argument(base)
        })
    }

    fn ok_or_not_found(self, msg: impl IntoOptionalString) -> Result<T, Status> {
        self.ok_or_else(|| {
            let mut base = "not found".to_string();

            if let Some(msg) = msg.as_ref() {
                base += &format!(": {msg}");
            }

            Status::not_found(base)
        })
    }
}

fn ok_or_invalid_argument<T, E: Error>(result: Result<T, E>, msg: impl IntoOptionalString) -> Result<T, Status> {
    result.map_err(|e| {
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

fn ok_or_not_found<T, E: Error>(result: Result<T, E>, msg: impl IntoOptionalString) -> Result<T, Status> {
    result.map_err(|e| {
        let mut base = "not found".to_string();

        if let Some(msg) = msg.as_ref() {
            base += &format!(": {msg}");
        }

        if cfg!(debug_assertions) {
            base += &format!("; {e}")
        }

        Status::not_found(base)
    })
}

impl<T, E: Error> ServiceErrorExt<T> for Result<T, E> {
    default fn ok_or_invalid_argument(self, msg: impl IntoOptionalString) -> Result<T, Status> {
        ok_or_invalid_argument(self, msg)
    }

    fn ok_or_not_found(self, msg: impl IntoOptionalString) -> Result<T, Status> {
        ok_or_not_found(self, msg)
    }
}

impl<T, E: Error> ServiceErrorExt<T> for Result<Option<T>, E> {
    fn ok_or_invalid_argument(self, msg: impl IntoOptionalString) -> Result<T, Status> {
        ok_or_invalid_argument(self, &msg)?
            .ok_or_invalid_argument(&msg)
    }

    fn ok_or_not_found(self, msg: impl IntoOptionalString) -> Result<T, Status> {
        ok_or_not_found(self, &msg)?
            .ok_or_not_found(msg)
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
