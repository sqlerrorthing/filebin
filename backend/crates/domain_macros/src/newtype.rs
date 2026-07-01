use proc_macro2::{Ident, Span};
use std::iter::chain;
use syn::Type;

macro_rules! idents {
    ($($ident:ident),* $(,)?) => {
        vec![
            $(
                Ident::new(stringify!($ident), Span::call_site())
            ),*
        ]
    };
}

fn basic_idents() -> Vec<Ident> {
    idents![Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Deref]
}

pub struct NewtypeMeta {
    pub const_fn: bool,
    pub derives: Vec<Ident>,
    pub derive_value_type: bool,
}

impl NewtypeMeta {
    fn number() -> Self {
        Self {
            const_fn: true,
            derives: chain(basic_idents(), idents![Copy, Display, FromStr, Hash]).collect(),
            derive_value_type: true,
        }
    }

    fn tinystr() -> Self {
        Self {
            const_fn: true,
            derives: chain(basic_idents(), idents![Display, FromStr]).collect(),
            derive_value_type: false,
        }
    }

    fn other() -> Self {
        Self {
            const_fn: false,
            derives: basic_idents(),
            derive_value_type: false
        }
    }

    fn uuid() -> Self {
        // cuz this is the same
        Self::number()
    }

    pub fn for_type(ty: &Type) -> Option<Self> {
        macro_rules! find {
            (
                in $ident_str:expr;
                $($($args:expr),* => $self:expr);*
                $(, default => $other:expr)?
            ) => {
                {
                    $(
                        if [$($args),*].iter().any(|&t| $ident_str == t) {
                            return Some($self)
                        }
                    )*

                    $($other)?
                }
            };
        }

        if let Type::Path(type_path) = ty
            && let Some(segment) = type_path.path.segments.last()
        {
            let ident_str = segment.ident.to_string();
            return find! {
                in &ident_str;

                "i64", "i32", "u64", "u32", "f64", "usize", "i16"
                    => Self::number();

                "Uuid", "uuid"
                    => Self::uuid();

                "TinyAsciiStr", "TinyStr"
                    => Self::tinystr(),

                default => Some(Self::other())
            }
        }

        None
    }
}
