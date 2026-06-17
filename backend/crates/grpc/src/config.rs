use pastey::paste;
use config_impl::{ConfigError, FileFormat};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::env::var;
use std::sync::LazyLock;
use std::time::Duration;
pub use secrecy;

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

config! {
    #[derive(Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub Config {
        pub jwt: {
            #[serde(with = "humantime_serde")]
            pub expires: Duration,
            pub secret: SecretString
        },
        pub db: {
            pub postgres_url: String
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
