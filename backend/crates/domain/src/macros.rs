macro_rules! tiny_str_sea_orm_derive {
    ($ty:ident($tinystr:ident)) => {
        impl From<$ty> for Value {
            fn from(source: $ty) -> Self {
                source.into_inner().as_str().into()
            }
        }
        #[automatically_derived]
        impl sea_orm::TryGetable for $ty {
            fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
                <String as sea_orm::TryGetable>::try_get_by(res, idx)
                    .map(|v| $ty::new($tinystr::try_from_str(&v).unwrap()))
            }
        }
        #[automatically_derived]
        impl sea_orm::sea_query::ValueType for $ty {
            fn try_from(v: Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
                <String as sea_orm::sea_query::ValueType>::try_from(v)
                    .map(|v| $ty::new($tinystr::try_from_str(&v).unwrap()))
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
}

pub(crate) use tiny_str_sea_orm_derive;