use serde::Deserialize;
use crate::macros::config;

mod macros;

config! {
    #[derive(Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub Config {
        pub jwt: {
            pub secret: String
        },
        pub db: {
            pub url: String
        },
        pub redis: {
            pub url: String
        },
    }
}
