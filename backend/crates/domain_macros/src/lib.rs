mod args;
mod generator;
mod newtype;

use crate::args::Args;
use crate::generator::Generator;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{Data, DeriveInput, Error, parse_macro_input, DataStruct, Fields};

#[proc_macro_derive(Model, attributes(model))]
pub fn model(input: TokenStream) -> TokenStream {
    let crate_root = match crate_name("domain") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
        Err(_) => quote!(::domain),
    };
    
    let input = parse_macro_input!(input as DeriveInput);
    let model_args = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("model"))
        .map(|attr| {
            attr.parse_args::<Args>()
        });
    
    let model_args = match model_args {
        Some(Ok(a)) => a,
        Some(Err(e)) => return e.to_compile_error().into(),
        None => Args::default()
    };

    match input.data.clone() {
        Data::Struct(DataStruct { fields: Fields::Named(named), .. }) => Generator::new(model_args, input, named, crate_root)
            .generate()
            .unwrap_or_else(|f| f.to_compile_error())
            .into(),
        _ => Error::new(Span::call_site(), "only named structs are supported")
            .to_compile_error()
            .into(),
    }
}
