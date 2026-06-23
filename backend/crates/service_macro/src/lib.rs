//! A proc macro `service` to define service trait

mod args;
mod expand;
mod kw;

use quote::quote;
use proc_macro::{TokenStream};
use syn::{parse_macro_input, ItemTrait};
use crate::args::Args;
use crate::expand::expand;

/// This supports attrs like `?Send` and `?Sync`.
///
/// This won't add this to trait bounds if provided
///
/// * Use `#[result]` to wrap return value into `Result<..., Self::Error>`
/// * Use `#[result(Err)` where `Err` is your business error,
///   and it will wrap it in `Result<..., ServiceError<Err, Self::Error>>`
#[proc_macro_attribute]
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as Args);
    let mut item = parse_macro_input!(item as ItemTrait);
    expand(&mut item, args);
    TokenStream::from(quote!(#item))
}
