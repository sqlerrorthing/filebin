pub(crate) use pastey;

macro_rules! config {
    (@gen_struct [$(#[$struct_attrs:meta])*] $vis:vis $field:ident { $($sub:tt)* }) => {
        $crate::macros::pastey::paste! {
            $crate::config! {
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
        $crate::config!(@gen_struct [$(#[$struct_attrs])*] $vis $field { $($sub)* });
        $crate::config!(@parse $name {
            struct_attrs: [$(#[$struct_attrs])*]
            fields: {
                $($out)*
                $(#[$attrs])*
                $vis $field: $crate::macros::pastey::paste!{[<$field:camel>]},
            }
        } $($rest)*);
    };

    (@parse $name:ident { struct_attrs: [$(#[$struct_attrs:meta])*] fields: { $($out:tt)* } }
        $(#[$attrs:meta])* $vis:vis $field:ident : { $($sub:tt)* }
    ) => {
        $crate::config!(@gen_struct [$(#[$struct_attrs])*] $vis $field { $($sub)* });
        $crate::config!(@parse $name {
            struct_attrs: [$(#[$struct_attrs])*]
            fields: {
                $($out)*
                $(#[$attrs])*
                $vis $field: $crate::macros::pastey::paste!{[<$field:camel>]},
            }
        });
    };

    (@parse $name:ident { struct_attrs: [$(#[$struct_attrs:meta])*] fields: { $($out:tt)* } }
        $(#[$attrs:meta])* $vis:vis $field:ident : $ty:ty , $($rest:tt)*
    ) => {
        $crate::config!(@parse $name {
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
        $crate::config!(@parse $name {
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
        $crate::config!(@parse $name { struct_attrs: [$(#[$struct_attrs])*] fields: {} } $($body)*);
    };

    (
        $(#[$struct_attrs:meta])*
        $($body:tt)*
    ) => {
        $crate::config!(@parse Struct { struct_attrs: [$(#[$struct_attrs])*] fields: {} } $($body)*);
    };

    (@parse $($rest:tt)*) => {
        compile_error!("Invalid syntax inside config! macro. Check for missing colons or commas.");
    };
}

pub(crate) use config;
