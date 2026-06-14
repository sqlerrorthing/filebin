use crate::args::Args;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Attribute, ItemTrait, Meta, ReturnType, Token, TraitItem, TraitItemFn, Type, TypeParamBound};
use syn::punctuated::Punctuated;

pub fn expand(input: &mut ItemTrait, args: Args) {
    modify_associated_type(input, args);

    for item in &mut input.items {
        if let TraitItem::Fn(method) = item {
            process_method(method, args)
        }
    }
}

fn process_method(method: &mut TraitItemFn, args: Args) {
    let mut current_output: Type = match &method.sig.output {
        ReturnType::Default => syn::parse_quote!(()),
        ReturnType::Type(_, ty) => *ty.clone(),
    };

    wrap_ret_with_error(&mut method.attrs, &mut current_output);
    wrap_ret_with_async_impl(&mut method.sig.asyncness, &mut current_output, args);
    method.sig.output = parse_quote! { -> #current_output };
}

fn wrap_ret_with_async_impl(is_async: &mut Option<Token![async]>, current_output: &mut Type, args: Args) {
    if is_async.take().is_none() {
        return
    }

    let mut future_bounds: Punctuated<TypeParamBound, Token![+]> = Punctuated::new();
    future_bounds.push(parse_quote! {
        ::core::future::Future<Output = #current_output>
    });

    if args.require_send {
        future_bounds.push(syn::parse_quote!(Send));
    }
    if args.require_sync {
        future_bounds.push(syn::parse_quote!(Sync));
    }

    *current_output = parse_quote! { impl #future_bounds };
}

fn wrap_ret_with_error(attrs: &mut Vec<Attribute>, current_output: &mut Type) {
    let require_attr_info = extract_and_remove_result_attr(attrs);

    if let Some(maybe_err_wrapper) = require_attr_info {
        if let Some(custom_error) = maybe_err_wrapper {
            *current_output = syn::parse_quote! {
                Result<#current_output, service::error::ServiceError<#custom_error, Self::Error>>
            };
        } else {
            *current_output = syn::parse_quote! {
                Result<#current_output, Self::Error>
            };
        }
    }
}

fn extract_and_remove_result_attr(
    attrs: &mut Vec<Attribute>,
) -> Option<Option<TokenStream>> {
    let index = attrs
        .iter()
        .position(|attr| attr.path().is_ident("result"))?;
    let attr = attrs.remove(index);

    match attr.meta {
        Meta::Path(_) => Some(None),
        Meta::List(meta_list) => {
            let tokens = meta_list.tokens;
            Some(Some(tokens))
        }
        Meta::NameValue(_) => Some(None),
    }
}

fn modify_associated_type(input: &mut ItemTrait, args: Args) {
    for item in &mut input.items {
        if let TraitItem::Type(assoc_type) = item {
            if args.require_send {
                assoc_type.bounds.push(syn::parse_quote!(Send));
            }
            if args.require_sync {
                assoc_type.bounds.push(syn::parse_quote!(Sync));
            }
        }
    }
}
