//! A proc macro `service` to define service trait

mod args;
mod expand;
mod generator;

use crate::args::{Args, AttrArgs};
use crate::expand::expand;
use crate::generator::DynGeneratorContext;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Error, ItemTrait, parse_macro_input};

/// This supports attrs like `?(...)` and `dynamic`.
///
/// This won't add this to trait bounds if provided
///
/// * Use `#[result]` to wrap return value into `Result<..., Self::Error>`
/// * Use `#[result(Err)` where `Err` is your business error,
///   and it will wrap it in `Result<..., ServiceError<Err, Self::Error>>`
#[proc_macro_attribute]
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    let crate_root = match crate_name("service") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
        Err(_) => quote!(::service),
    };

    let args = parse_macro_input!(attr as AttrArgs);
    let args = Args {
        requires: args.requires,
        dynamic_dispatch: args.dynamic_dispatch,
        service_crate_root: crate_root,
    };

    let mut item = parse_macro_input!(item as ItemTrait);
    expand(&mut item, &args);

    let generated = DynGeneratorContext::new(args, item.clone())
        .gen_dyn_impl()
        .unwrap_or_else(Error::into_compile_error);

    TokenStream::from(quote!(
        #item
        #generated
    ))
}
