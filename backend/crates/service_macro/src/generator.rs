use crate::args::Args;
use derive_new::new;
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};
use std::collections::{BTreeMap, HashMap};
use std::ops::ControlFlow;
use syn::visit_mut::VisitMut;
use syn::{
    GenericArgument, ItemTrait, PathArguments, PathSegment, Result, ReturnType, Token, TraitBound,
    TraitItem, TraitItemFn, Type, TypeImplTrait, TypeParamBound, TypePath, parse_quote,
};

#[derive(Default, Clone, Copy, Debug)]
enum Boxing {
    /// Box::pin
    Pin,
    /// Box::new
    #[default]
    Normal,
}

impl Boxing {
    fn into_box<X: ToTokens>(self, x: &X) -> TokenStream {
        match self {
            Boxing::Pin => {
                quote! {
                    ::std::boxed::Box::pin(#x) as ::std::pin::Pin<::std::boxed::Box<_>>
                }
            }
            Boxing::Normal => {
                quote! {
                    ::std::boxed::Box::new(#x) as ::std::boxed::Box<_>
                }
            }
        }
    }
}

#[derive(Default, Debug)]
struct TransformMeta {
    /// Is return type is `Result`
    is_result: bool,
    /// Is need to map the result to box
    into_box: Option<Boxing>,
    map_err_into: bool,
}

#[derive(Debug)]
struct ParsedAssocType {
    ident: Ident,
    erased_ty: Type,
    into_box: Option<Boxing>,
    map_err_into: bool,
    alias_def: Option<TokenStream>
}

#[derive(Debug)]
struct ParsedMethod {
    /// Original method name (for example `do_work`)
    orig_ident: Ident,
    /// Dyn method name (for example, `erased_do_work`)
    dyn_ident: Ident,
    /// Fn arguments (including `self`)
    inputs: syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    orig_ret: ReturnType,
    /// Erased return type
    ret: ReturnType,
    async_kw: Option<Token![async]>,
    meta: TransformMeta,
}

#[derive(new)]
pub(crate) struct DynGeneratorContext {
    args: Args,
    trait_def: ItemTrait,
}

impl DynGeneratorContext {
    pub(crate) fn gen_dyn_impl(self) -> Result<TokenStream> {
        if !self.args.dynamic_dispatch {
            return Ok(TokenStream::new());
        }

        let assoc_types = self.parse_assoc_types()?;
        let methods = self.parse_methods(&assoc_types)?;

        let trait_aliases: Vec<_> = assoc_types
            .values()
            .filter_map(|assoc| assoc.alias_def.clone())
            .collect();

        let module = format_ident!("__dyn_{}", self.trait_def.ident);
        let vis = self.trait_def.vis.clone();
        let trait_name = format_ident!("Dyn{}", self.trait_def.ident);

        let mut ctx = ParsedGeneratorContext {
            ctx: self,
            methods,
            assoc_types,
        };

        let dispatch_trait = ctx.gen_dyn_trait()?;
        let impl_dynamic_for_service = ctx.gen_impl_dynamic_for_service()?;
        let impl_service_for_dynamic = ctx.gen_impl_service_for_dynamic()?;

        Ok(quote! {
            #[doc(hidden)]
            #[allow(non_snake_case, reason = "we keep original structure name which is probably CamelCase")]
            mod #module {
                use super::*;

                #(#trait_aliases)*

                #dispatch_trait
                #impl_dynamic_for_service
                #impl_service_for_dynamic
            }
            #vis use #module::Dynamic as #trait_name;
        })
    }

    fn erase_ret(
        &self,
        ret: &mut ReturnType,
        meta: &mut TransformMeta,
        assoc_types: &HashMap<Ident, ParsedAssocType>,
    ) {
        RetTypeDynSignatureEraser { assoc_types, meta }.visit_return_type_mut(ret);
    }

    fn parse_assoc_types(&self) -> Result<HashMap<Ident, ParsedAssocType>> {
        let mut result = HashMap::new();
        let root = &self.args.service_crate_root;

        for item in &self.trait_def.items {
            if let TraitItem::Type(decl) = item {
                let mut alias_def = None;

                let ident = decl.ident.clone();
                let bounds = &decl.bounds;

                let mut map_err_into = false;
                let mut into_box = None;
                let erased_ty: Type;

                if ident == "Error" {
                    map_err_into = true;
                    let mut has_send = false;
                    let mut has_sync = false;

                    for b in bounds.iter() {
                        if let TypeParamBound::Trait(tb) = b {
                            if tb.path.segments.last().is_some_and(|s| s.ident == "Send") {
                                has_send = true;
                            } else if tb.path.segments.last().is_some_and(|s| s.ident == "Sync") {
                                has_sync = true;
                            }
                        }
                    }

                    erased_ty = match (has_send, has_sync) {
                        (true, true) => parse_quote! { #root::error::DynamicSendSyncError },
                        (true, false) => parse_quote! { #root::error::DynamicSendError },
                        (false, true) => parse_quote! { #root::error::DynamicSyncError },
                        (false, false) => parse_quote! { #root::error::DynamicError },
                    };
                } else {
                    let pinned = bounds.iter().any(|b| {
                        if let TypeParamBound::Trait(tb) = b {
                            tb.path.segments.last().is_some_and(|s| s.ident == "Stream")
                        } else {
                            false
                        }
                    });

                    let alias_ident = format_ident!("__{}", ident);

                    alias_def = Some(quote! {
                        pub trait #alias_ident: #bounds {}
                        impl<__T: ?Sized + #bounds> #alias_ident for __T {}
                    });

                    if pinned {
                        into_box = Some(Boxing::Pin);
                        erased_ty = parse_quote! {
                            ::core::pin::Pin<::std::boxed::Box<dyn #alias_ident>>
                        };
                    } else {
                        into_box = Some(Boxing::Normal);
                        erased_ty = parse_quote! {
                            ::std::boxed::Box<dyn #alias_ident>
                        };
                    }
                }

                result.insert(
                    ident.clone(),
                    ParsedAssocType {
                        ident,
                        erased_ty,
                        into_box,
                        map_err_into,
                        alias_def
                    },
                );
            }
        }

        // todo: this works but call conversations isn't satisfied
        let type_map = result
            .iter()
            .map(|(i, parsed)| (i.clone(), parsed.erased_ty.clone()))
            .collect();

        let mut eraser = AssocTypeEraser {
            type_map: &type_map,
        };

        for parsed in result.values_mut() {
            eraser.visit_type_mut(&mut parsed.erased_ty);
        }

        Ok(result)
    }

    fn parse_methods(
        &self,
        assoc_types: &HashMap<Ident, ParsedAssocType>,
    ) -> Result<Vec<ParsedMethod>> {
        let mut parsed = Vec::new();

        for item in &self.trait_def.items {
            if let TraitItem::Fn(orig_fn) = item {
                let orig_ident = orig_fn.sig.ident.clone();
                let dyn_ident = format_ident!("erased_{}", orig_ident);
                let inputs = orig_fn.sig.inputs.clone();

                let (is_async, mut ret) = self.sugar_async(orig_fn)?;
                let mut meta = TransformMeta::default();

                if let ReturnType::Type(_, ty) = &ret
                    && let Type::Path(tp) = &**ty
                    && tp.path.segments.last().is_some_and(|s| s.ident == "Result")
                {
                    meta.is_result = true;
                }

                self.erase_ret(&mut ret, &mut meta, assoc_types);

                parsed.push(ParsedMethod {
                    orig_ident,
                    dyn_ident,
                    inputs,
                    orig_ret: orig_fn.sig.output.clone(),
                    ret,
                    async_kw: is_async.then_some(parse_quote!(async)),
                    meta,
                });
            }
        }

        Ok(parsed)
    }

    fn sugar_async(&self, orig_fn: &TraitItemFn) -> Result<(bool, ReturnType)> {
        if orig_fn.sig.asyncness.is_some() {
            return Ok((true, orig_fn.sig.output.clone()));
        }

        if let ReturnType::Type(_, ty) = &orig_fn.sig.output
            && let Type::ImplTrait(TypeImplTrait { bounds, .. }) = &**ty
        {
            for bound in bounds {
                if let TypeParamBound::Trait(TraitBound { path, .. }) = bound
                    && path.segments.last().is_some_and(|s| s.ident == "Future")
                    && let PathArguments::AngleBracketed(args) =
                        &path.segments.last().unwrap().arguments
                {
                    for arg in &args.args {
                        if let GenericArgument::AssocType(assoc_type) = arg
                            && assoc_type.ident == "Output"
                        {
                            let output_ty = &assoc_type.ty;
                            return Ok((true, parse_quote! { -> #output_ty }));
                        }
                    }
                }
            }
        }

        Ok((false, orig_fn.sig.output.clone()))
    }
}

#[derive(new)]
struct ParsedGeneratorContext {
    ctx: DynGeneratorContext,
    methods: Vec<ParsedMethod>,
    assoc_types: HashMap<Ident, ParsedAssocType>,
}

impl ParsedGeneratorContext {
    fn gen_dyn_trait(&mut self) -> Result<TokenStream> {
        let header = self.gen_header()?;
        let tests = self.gen_tests()?;

        let sigs = self.methods.iter().map(|m| {
            let dyn_ident = &m.dyn_ident;
            let inputs = &m.inputs;
            let ret = &m.ret;
            let async_kw = m.async_kw;

            quote! {
                #async_kw fn #dyn_ident(#inputs) #ret;
            }
        });

        Ok(quote! {
            #header {
                #(#sigs)*
            }

            #tests
        })
    }

    fn gen_tests(&mut self) -> Result<TokenStream> {
        let (impl_generics, ty_generics, _) = &self.ctx.trait_def.generics.split_for_impl();
        Ok(quote! {
            mod tests {
                use super::*;

                fn check_dyn_compat #impl_generics () {
                    let _is_dyn_compat: Option<&dyn Dynamic #ty_generics> = None;
                }
            }
        })
    }

    fn gen_header(&mut self) -> Result<TokenStream> {
        let async_trait = self.async_trait_macro();

        let (_, generics, where_clause) = self.ctx.trait_def.generics.split_for_impl();
        let traits = &self.ctx.trait_def.supertraits;
        let supertraits = (!traits.is_empty()).then_some(quote!(: #traits));

        Ok(quote! {
            #async_trait
            pub trait Dynamic #generics #supertraits #where_clause
        })
    }

    fn async_trait_macro(&self) -> Option<TokenStream> {
        let root = &self.ctx.args.service_crate_root;
        let has_async = self.methods.iter().any(|m| m.async_kw.is_some());
        has_async.then_some(quote!(#[#root::async_trait::async_trait]))
    }

    fn gen_impl_dynamic_for_service(&mut self) -> Result<TokenStream> {
        let orig_name = &self.ctx.trait_def.ident;
        let async_trait = self.async_trait_macro();

        let mut generics = self.ctx.trait_def.generics.clone();
        let (_, ty_generics, _) = self.ctx.trait_def.generics.split_for_impl();

        let service_param: syn::GenericParam = syn::parse_quote!(__Service);
        generics.params.push(service_param);

        let where_clause = generics.make_where_clause();

        where_clause.predicates.push(syn::parse_quote!(
            __Service: #orig_name #ty_generics
        ));

        for item in &self.ctx.trait_def.items {
            if let TraitItem::Type(assoc_type) = item {
                let assoc_ident = &assoc_type.ident;

                where_clause.predicates.push(syn::parse_quote!(
                    <__Service as #orig_name #ty_generics>::#assoc_ident: 'static
                ));
            }
        }

        let (impl_generics, _, where_clause) = generics.split_for_impl();
        let body = self.gen_impl_dynamic_for_service_body()?;

        Ok(quote! {
            #async_trait
            impl #impl_generics Dynamic #ty_generics for __Service #where_clause {
                #(#body)*
            }
        })
    }

    fn gen_impl_dynamic_for_service_body(&self) -> Result<impl Iterator<Item = TokenStream>> {
        let root = self.ctx.args.service_crate_root.clone();
        let sigs = self.methods.iter().map(move |m| {
            let service = &self.ctx.trait_def.ident;
            let orig_ident = &m.orig_ident;
            let dyn_ident = &m.dyn_ident;
            let inputs = &m.inputs;
            let ret = &m.ret;
            let async_kw = m.async_kw;

            let has_receiver = inputs
                .first()
                .is_some_and(|arg| matches!(arg, syn::FnArg::Receiver(_)));

            let args = inputs.iter().filter_map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg
                    && let syn::Pat::Ident(pat_ident) = &*pat_type.pat
                {
                    let ident = &pat_ident.ident;
                    return Some(quote!(#ident));
                }

                None
            });

            let await_call = async_kw.map(|_| quote!(.await));

            let call = if has_receiver {
                quote!(#service::#orig_ident(self, #(#args),*))
            } else {
                quote!(#service::#orig_ident(#(#args),*))
            };

            let mut res_expr = quote!(#call #await_call);

            if m.meta.is_result {
                if let Some(boxing) = m.meta.into_box {
                    let ident = format_ident!("x");
                    let boxed = boxing.into_box(&ident);
                    res_expr = quote!(#res_expr.map(|#ident| #boxed))
                }

                if m.meta.map_err_into {
                    res_expr = quote!(#res_expr.map_err(#root::error::FromError::from_error));
                }
            } else {
                if let Some(boxing) = m.meta.into_box {
                    res_expr = boxing.into_box(&res_expr);
                }
            }

            quote! {
                #async_kw fn #dyn_ident(#inputs) #ret {
                    #res_expr
                }
            }
        });

        Ok(sigs)
    }

    fn gen_impl_service_for_dynamic(&self) -> Result<TokenStream> {
        let service = &self.ctx.trait_def.ident;
        let assoc = self.gen_impl_assoc_types()?;
        let methods = self.gen_impl_service_for_dynamic_methods()?;

        let mut generics = self.ctx.trait_def.generics.clone();

        {
            let where_clause = generics.make_where_clause();

            for param in &self.ctx.trait_def.generics.params {
                if let syn::GenericParam::Type(type_param) = param {
                    let ident = &type_param.ident;
                    where_clause.predicates.push(parse_quote!(#ident: 'static));
                }
            }
        }

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        Ok(quote! {
            impl #impl_generics #service #ty_generics for dyn Dynamic #ty_generics #where_clause {
                #(#assoc)*
                #(#methods)*
            }
        })
    }

    fn gen_impl_service_for_dynamic_methods(&self) -> Result<impl Iterator<Item = TokenStream>> {
        Ok(self.methods.iter().map(|m| {
            let orig_ident = &m.orig_ident;
            let orig_ret = &m.orig_ret;

            let dyn_ident = &m.dyn_ident;
            let inputs = &m.inputs;

            let args = inputs.iter().filter_map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg
                    && let syn::Pat::Ident(pat_ident) = &*pat_type.pat
                {
                    return Some(quote!(#pat_ident));
                }
                None
            });

            let has_receiver = inputs
                .first()
                .is_some_and(|arg| matches!(arg, syn::FnArg::Receiver(_)));

            let mut call = if has_receiver {
                quote!(Dynamic::#dyn_ident(self, #(#args),*))
            } else {
                quote!(Dynamic::#dyn_ident(#(#args),*))
            };

            if m.async_kw.is_some() {
                call = quote!(#call.await)
            }

            let block = if m.async_kw.is_some() {
                quote! {
                    async move {
                        #call
                    }
                }
            } else {
                quote! {
                    #call
                }
            };

            quote! {
                fn #orig_ident(#inputs) #orig_ret {
                    #block
                }
            }
        }))
    }

    fn gen_impl_assoc_types(&self) -> Result<impl Iterator<Item = TokenStream>> {
        Ok(self.ctx.trait_def.items.iter().filter_map(|x| {
            if let TraitItem::Type(decl) = x
                && let Some(parsed) = self.assoc_types.get(&decl.ident)
            {
                let ident = &parsed.ident;
                let erased_ty = &parsed.erased_ty;
                Some(quote! {
                    type #ident = #erased_ty;
                })
            } else {
                None
            }
        }))
    }
}

macro_rules! erase {
    ($i:ident, |$x:ident| => $block:block) => {
        if let Type::Path(TypePath { qself, path }) = $i
            && qself.is_none()
            && path.segments.first().is_some_and(|s| s.ident == "Self")
            && let Some($x) = path.segments.get(1)
        $block
    };
}

struct AssocTypeEraser<'a> {
    type_map: &'a HashMap<Ident, Type>,
}

impl VisitMut for AssocTypeEraser<'_> {
    fn visit_type_mut(&mut self, i: &mut Type) {
        erase!(i, |assoc_segment| => {
            if let Some(erased_ty) = self.type_map.get(&assoc_segment.ident) {
                *i = erased_ty.clone();
                return;
            }
        });

        syn::visit_mut::visit_type_mut(self, i);
    }
}

struct RetTypeDynSignatureEraser<'a> {
    meta: &'a mut TransformMeta,
    assoc_types: &'a HashMap<Ident, ParsedAssocType>,
}

impl VisitMut for RetTypeDynSignatureEraser<'_> {
    fn visit_type_mut(&mut self, i: &mut Type) {
        erase!(i, |assoc_segment| => {
            let assoc_ident = &assoc_segment.ident;

            if let Some(parsed) = self.assoc_types.get(assoc_ident) {
                self.meta.map_err_into = parsed.map_err_into;
                if parsed.into_box.is_some() {
                    self.meta.into_box = parsed.into_box;
                }
                *i = parsed.erased_ty.clone();
                return;
            }
        });

        syn::visit_mut::visit_type_mut(self, i);
    }
}
