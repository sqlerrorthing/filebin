use crate::args::{Args, Requires};
use proc_macro2::TokenStream;
use syn::punctuated::Punctuated;
use syn::{
    Attribute, ItemTrait, Meta, ReturnType, Token, TraitItem, TraitItemFn, TraitItemType, Type,
    TypeParamBound, parse_quote,
};

pub fn expand(input: &mut ItemTrait, args: &Args) {
    // todo: uncomment
    push_auto_impls(&args.service_crate_root, input);
    modify_associated_type(input, args.requires);
    expand_supertraits(args.requires, &mut input.supertraits);
    for item in &mut input.items {
        if let TraitItem::Fn(method) = item {
            process_method(method, args.requires)
        }
    }
}

fn push_auto_impls(root: &TokenStream, input: &mut ItemTrait) {
    input.attrs.push(parse_quote! {
        #[#root::auto_impl::auto_impl(&, Box, Arc)]
    })
}

fn process_method(method: &mut TraitItemFn, requires: Requires) {
    let mut ret = match &method.sig.output {
        ReturnType::Default => parse_quote!(()),
        ReturnType::Type(_, ty) => ty.clone(),
    };

    wrap_ret(method, &mut ret, requires)
}

fn wrap_ret(method: &mut TraitItemFn, out: &mut Type, requires: Requires) {
    wrap_ret_with_error(&mut method.attrs, out);
    wrap_ret_with_async_impl(&mut method.sig.asyncness, out, requires);
    method.sig.output = parse_quote! { -> #out };
}

fn wrap_ret_with_async_impl(
    is_async: &mut Option<Token![async]>,
    current_output: &mut Type,
    mut requires: Requires,
) {
    if is_async.take().is_none() {
        return;
    }

    let mut future_bounds: Punctuated<TypeParamBound, Token![+]> = Punctuated::new();
    future_bounds.push(parse_quote! {
        ::core::future::Future<Output = #current_output>
    });

    requires -= Requires::DEBUG | Requires::SYNC | Requires::STATIC;
    expand_supertraits(requires, &mut future_bounds);
    *current_output = parse_quote! { impl #future_bounds };
}

fn expand_supertraits<P: Default>(requires: Requires, bounds: &mut Punctuated<TypeParamBound, P>) {
    if requires.contains(Requires::SEND) {
        bounds.push(parse_quote!(Send));
    }

    if requires.contains(Requires::SYNC) {
        bounds.push(parse_quote!(Sync));
    }

    if requires.contains(Requires::DEBUG) {
        bounds.push(parse_quote!(std::fmt::Debug));
    }

    if requires.contains(Requires::STATIC) {
        bounds.push(parse_quote!('static));
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

fn extract_and_remove_result_attr(attrs: &mut Vec<Attribute>) -> Option<Option<TokenStream>> {
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

fn modify_associated_type(input: &mut ItemTrait, mut requires: Requires) {
    for item in &mut input.items {
        if let TraitItem::Type(assoc_type) = item {
            requires -= Requires::STATIC;
            if modify_error_associated_type(assoc_type) {
                requires -= Requires::DEBUG;
            }
            
            expand_supertraits(requires, &mut assoc_type.bounds);
        }
    }
}

fn modify_error_associated_type(input: &mut TraitItemType) -> bool {
    if input.ident == "Error" {
        input.bounds.extend::<[TypeParamBound; 2]>([
            parse_quote!(::core::error::Error),
            parse_quote!('static),
        ]);
        true
    } else {
        false
    }
}
