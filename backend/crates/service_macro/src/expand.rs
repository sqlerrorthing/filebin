use crate::args::Args;
use proc_macro2::TokenStream;
use syn::{parse_quote, Attribute, ItemTrait, Meta, ReturnType, Token, TraitItem, TraitItemFn, TraitItemType, Type, TypeParamBound};
use syn::punctuated::Punctuated;

pub fn expand(input: &mut ItemTrait, args: Args) {
    push_auto_impls(input);
    modify_associated_type(input, args);
    expand_supertraits(args, &mut input.supertraits);
    for item in &mut input.items {
        if let TraitItem::Fn(method) = item {
            process_method(method, args)
        }
    }
}

fn push_auto_impls(input: &mut ItemTrait) {
    input.attrs.push(
        parse_quote! {
            #[service::auto_impl::auto_impl(&, Box, Arc)]
        }
    )
}

fn process_method(method: &mut TraitItemFn, args: Args) {
    let mut current_output: Type = match &method.sig.output {
        ReturnType::Default => parse_quote!(()),
        ReturnType::Type(_, ty) => *ty.clone(),
    };

    wrap_ret_with_error(&mut method.attrs, &mut current_output);
    wrap_ret_with_async_impl(&mut method.sig.asyncness, &mut current_output, args);
    method.sig.output = parse_quote! { -> #current_output };
}

fn wrap_ret_with_async_impl(is_async: &mut Option<Token![async]>, current_output: &mut Type, mut args: Args) {
    if is_async.take().is_none() {
        return
    }

    let mut future_bounds: Punctuated<TypeParamBound, Token![+]> = Punctuated::new();
    future_bounds.push(parse_quote! {
        ::core::future::Future<Output = #current_output>
    });

    args.require_debug = false;
    args.require_sync = false;
    expand_supertraits(args, &mut future_bounds);
    *current_output = parse_quote! { impl #future_bounds };
}

fn expand_supertraits<P: Default>(args: Args, bounds: &mut Punctuated<TypeParamBound, P>) {
    if args.require_send {
        bounds.push(parse_quote!(Send));
    }

    if args.require_sync {
        bounds.push(parse_quote!(Sync));
    }

    if args.require_debug {
        bounds.push(parse_quote!(std::fmt::Debug));
    }
}

fn wrap_ret_with_error(attrs: &mut Vec<Attribute>, current_output: &mut Type) {
    let require_attr_info = extract_and_remove_result_attr(attrs);

    if let Some(maybe_err_wrapper) = require_attr_info {
        if let Some(custom_error) = maybe_err_wrapper {
            *current_output = parse_quote! {
                ::core::result::Result<#current_output, service::error::ServiceError<#custom_error, Self::Error>>
            };
        } else {
            *current_output = parse_quote! {
                ::core::result::Result<#current_output, Self::Error>
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
            expand_supertraits(args, &mut assoc_type.bounds);
            modify_error_associated_type(assoc_type);
        }
    }
}

fn modify_error_associated_type(input: &mut TraitItemType) {
    if input.ident == "Error" {
        input.bounds.extend::<[TypeParamBound; 2]>([
            parse_quote!(::core::error::Error),
            parse_quote!('static),
        ]);
    }
}
