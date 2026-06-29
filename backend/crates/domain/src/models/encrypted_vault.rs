use tinystr::TinyAsciiStr;
use crate::macros::tiny_str_sea_orm_derive;
use nutype::nutype;
use serde::{Deserialize, Serialize};
use sea_orm::entity::prelude::DeriveValueType;
use strum_macros::EnumIter;

#[nutype(
    const_fn,
    derive(
        Debug,
        PartialEq,
        Eq,
        Copy,
        Clone,
        Serialize,
        Deserialize,
        Hash,
        Deref,
        Display,
        FromStr
    ),
    derive_unchecked(DeriveValueType)
)]
pub struct Id(i32);

#[nutype(
    const_fn,
    derive(
        Debug,
        PartialEq,
        Eq,
        Copy,
        Clone,
        Serialize,
        Deserialize,
        Hash,
        Deref,
        Display,
        FromStr
    ),
    derive_unchecked(DeriveValueType)
)]
pub struct Version(i16);

macro_rules! b64_encoded_exact_size {
    (
        $($ident:ident($len:literal))*
    ) => {
        $(
            #[nutype(
                derive(
                    Debug,
                    PartialEq,
                    Eq,
                    Clone,
                    Serialize,
                    Deserialize,
                    Hash,
                    Deref,
                    Display,
                    FromStr,
                ),
                validate(predicate = |s| {
                    use base64::{engine::general_purpose, Engine as _};

                    s.len() == $len
                    && general_purpose::STANDARD.decode(s.as_str()).is_ok()
                })
            )]
            pub struct $ident(TinyAsciiStr<$len>);

            tiny_str_sea_orm_derive!($ident as $len: |s|
                {
                    Self::try_new(s)
                        .map_err(|_| TryGetError::DbErr(DbErr::Type(concat!("expected a base64-encoded string of ", $len, " character from the database").to_string())))?
                },
                {
                    Self::try_new(s).map_err(|_| sea_orm::sea_query::ValueTypeErr)?
                }
            );
        )*
    };
}

b64_encoded_exact_size!(IV(16) Tag(24));

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
pub enum EncryptionAlgo {
    Aes256Gcm
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Model {
    pub id: Id,
    pub iv: IV,
    pub tag: Tag,
    pub var: Version,
    pub algo: EncryptionAlgo
}
