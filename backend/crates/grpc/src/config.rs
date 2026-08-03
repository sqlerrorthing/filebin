use byte_unit::Byte;
use config_impl::{ConfigError, FileFormat};
use pastey::paste;
pub use secrecy;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::env::var;
use std::sync::LazyLock;
use std::time::Duration;
use tower_http::cors::AllowOrigin;
use wildmatch::WildMatch;

macro_rules! config {
    (@gen_struct [$(#[$struct_attrs:meta])*] $vis:vis $field:ident { $($sub:tt)* }) => {
        paste! {
            config! {
                $(#[$struct_attrs])*
                $vis [<$field:camel>] { $($sub)* }
            }
        }
    };

    (@parse $name:ident { struct_attrs: [$(#[$struct_attrs:meta])*] fields: { $($out:tt)* } }) => {
        #[derive(Debug, Clone)]
        $(#[$struct_attrs])*
        pub struct $name {
            $($out)*
        }
    };

    (@parse $name:ident { struct_attrs: [$(#[$struct_attrs:meta])*] fields: { $($out:tt)* } }
        $(#[$attrs:meta])* $vis:vis $field:ident : { $($sub:tt)* } , $($rest:tt)*
    ) => {
        config!(@gen_struct [$(#[$struct_attrs])*] $vis $field { $($sub)* });
        config!(@parse $name {
            struct_attrs: [$(#[$struct_attrs])*]
            fields: {
                $($out)*
                $(#[$attrs])*
                $vis $field: paste!{[<$field:camel>]},
            }
        } $($rest)*);
    };

    (@parse $name:ident { struct_attrs: [$(#[$struct_attrs:meta])*] fields: { $($out:tt)* } }
        $(#[$attrs:meta])* $vis:vis $field:ident : { $($sub:tt)* }
    ) => {
        config!(@gen_struct [$(#[$struct_attrs])*] $vis $field { $($sub)* });
        config!(@parse $name {
            struct_attrs: [$(#[$struct_attrs])*]
            fields: {
                $($out)*
                $(#[$attrs])*
                $vis $field: paste!{[<$field:camel>]},
            }
        });
    };

    (@parse $name:ident { struct_attrs: [$(#[$struct_attrs:meta])*] fields: { $($out:tt)* } }
        $(#[$attrs:meta])* $vis:vis $field:ident : $ty:ty , $($rest:tt)*
    ) => {
        config!(@parse $name {
            struct_attrs: [$(#[$struct_attrs])*]
            fields: {
                $($out)*
                $(#[$attrs])*
                $vis $field: $ty,
            }
        } $($rest)*);
    };

    (@parse $name:ident { struct_attrs: [$(#[$struct_attrs:meta])*] fields: { $($out:tt)* } }
        $(#[$attrs:meta])* $vis:vis $field:ident : $ty:ty
    ) => {
        config!(@parse $name {
            struct_attrs: [$(#[$struct_attrs])*]
            fields: {
                $($out)*
                $(#[$attrs])*
                $vis $field: $ty,
            }
        });
    };

    (
        $(#[$struct_attrs:meta])*
        $vis:vis $name:ident { $($body:tt)* }
    ) => {
        config!(@parse $name { struct_attrs: [$(#[$struct_attrs])*] fields: {} } $($body)*);
    };

    (
        $(#[$struct_attrs:meta])*
        $($body:tt)*
    ) => {
        config!(@parse Struct { struct_attrs: [$(#[$struct_attrs])*] fields: {} } $($body)*);
    };

    (@parse $($rest:tt)*) => {
        compile_error!("Invalid syntax inside config! macro. Check for missing colons or commas.");
    };
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case", untagged)]
pub enum CorsOrigins {
    Single(String),
    List(Vec<String>),
}

impl From<&CorsOrigins> for AllowOrigin {
    fn from(value: &CorsOrigins) -> Self {
        fn match_origins(origins: &[impl AsRef<str>]) -> AllowOrigin {
            let origins: Vec<&str> = origins.iter().map(AsRef::as_ref).collect();

            if origins.iter().any(|o| o.contains(['?', '*'])) {
                let patterns: Vec<_> = origins.into_iter().map(WildMatch::new).collect();
                AllowOrigin::predicate(move |val, _| {
                    val.to_str().is_ok_and(|s| patterns.iter().any(|p| p.matches(s)))
                })
            } else {
                AllowOrigin::list(origins.into_iter().map(|o| o.parse().unwrap()))
            }
        }

        match value {
            CorsOrigins::Single(origin) if origin == "*" => AllowOrigin::mirror_request(),
            CorsOrigins::Single(origin) => match_origins(&[origin]),
            CorsOrigins::List(origins) => match_origins(origins),
        }
    }
}

config! {
    #[derive(Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub Config {
        pub cors: CorsOrigins,
        pub jwt: {
            #[serde(with = "humantime_serde")]
            pub expires: Duration,
            pub secret: SecretString,
        },
        pub caches: {
            #[serde(with = "humantime_serde")]
            pub folders: Duration,
            #[serde(with = "humantime_serde")]
            pub files: Duration,
        },
        pub redis: {
            pub url: String
        },
        pub rabbitmq: {
            pub url: Option<String>,
            pub exchange: String,
        },
        pub limits: {
            pub max_filesize: Byte,
            pub max_files_per_folder: u32
        },
        pub storage: {
            pub access_key: SecretString,
            pub secret_key: SecretString,
            pub bucket: String,
            pub region: String,
            pub endpoint_url: Option<String>,
            pub force_path_style: Option<bool>
        },
        pub db: {
            pub postgres_url: String,
        },
    }
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| validate_config(load_config().unwrap()));

fn validate_config(config: Config) -> Config {
    if config.jwt.secret.expose_secret().len() < 32 {
        panic!("jwt.secret is less than 32 symbols!")
    }

    config
}

fn load_config() -> Result<Config, ConfigError> {
    let mut builder = config_impl::Config::builder();

    if let Ok(cfg) = var("SERVER__CONFIG") {
        builder = builder.add_source(config_impl::File::new(&cfg, FileFormat::Yaml));
    }

    builder
        .add_source(config_impl::Environment::with_prefix("SERVER").separator("__"))
        .build()?
        .try_deserialize()
}
