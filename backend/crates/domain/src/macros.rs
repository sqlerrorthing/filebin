use sea_orm::{DbErr, TryGetError};

macro_rules! tiny_str_sea_orm_derive {
    ($ty:path as $len:literal: |$str:ident| $($construct:tt)*) => {
        #[allow(unused_imports)]
        const _: () = {
            use super::*;
            use sea_orm::entity::prelude::*;

            impl From<$ty> for Value {
                fn from(source: $ty) -> Self {
                    source.into_inner().as_str().into()
                }
            }

            #[automatically_derived]
            impl sea_orm::TryGetable for $ty {
                fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
                    let v = <String as sea_orm::TryGetable>::try_get_by(res, idx)?;
                    let $str = TinyAsciiStr::<$len>::try_from_str(&v)
                        .map_err(|_| TryGetError::DbErr(DbErr::Type("mismatch types".to_string())))?;

                    Ok(tiny_str_sea_orm_derive!(@gb $($construct)*))
                }
            }

            #[automatically_derived]
            impl sea_orm::sea_query::ValueType for $ty {
                fn try_from(v: Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
                    let v = <String as sea_orm::sea_query::ValueType>::try_from(v)?;
                    let $str = TinyAsciiStr::<$len>::try_from_str(&v)
                        .map_err(|_| sea_orm::sea_query::ValueTypeErr)?;

                    Ok(tiny_str_sea_orm_derive!(@tf $($construct)*))
                }
                fn type_name() -> String {
                    stringify!($ty).to_owned()
                }
                fn array_type() -> sea_orm::sea_query::ArrayType {
                    <String as sea_orm::sea_query::ValueType>::array_type()
                }
                fn column_type() -> ColumnType {
                    <String as sea_orm::sea_query::ValueType>::column_type()
                }
            }

            #[automatically_derived]
            impl sea_orm::sea_query::Nullable for $ty {
                fn null() -> Value {
                    Value::String(None)
                }
            }

            #[automatically_derived]
            impl sea_orm::IntoActiveValue<$ty> for $ty {
                fn into_active_value(self) -> sea_orm::ActiveValue<$ty> {
                    sea_orm::ActiveValue::Set(self)
                }
            }

            impl sea_orm::sea_query::postgres_array::NotU8 for $ty {}
        };
    };

    (@gb $b:block)           => { $b };
    (@gb $b:block, $_:block) => { $b };

    (@tf $b:block)           => { $b };
    (@tf $_:block, $b:block) => { $b };
}

pub(crate) use tiny_str_sea_orm_derive;
