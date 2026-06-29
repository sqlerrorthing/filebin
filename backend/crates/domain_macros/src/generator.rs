use crate::args::{Args, Input};
use crate::newtype::NewtypeMeta;
use derive_new::new;
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{DeriveInput, Error, FieldsNamed, Result};

#[derive(new)]
pub struct Generator {
    args: Args,
    input: DeriveInput,
    fields: FieldsNamed,
    _root: TokenStream,
}

impl Generator {
    fn check_struct_name(&self) -> Result<()> {
        (self.input.ident == "Model")
            .ok_or_else(|| Error::new(self.input.ident.span(), "Allowed only `Model` struct name"))
    }

    fn generate_newtypes(&self) -> Result<TokenStream> {
        let newtypes = self
            .args
            .newtypes
            .iter()
            .map(|t| {
                let meta = NewtypeMeta::for_type(&t.inner).ok_or(Error::new(
                    t.inner.span(),
                    "Not supported type for creating newtypes",
                ))?;

                let vis = &self.input.vis;
                let name = &t.name;
                let inner = &t.inner;

                let const_fn = meta.const_fn.then_some(quote!(const_fn));
                let derives = meta.derives;
                let derives = quote! {
                    derive(#(#derives),*)
                };

                let derive_value_type = meta.derive_value_type.then_some(quote!(derive_unchecked(
                    sea_orm::entity::prelude::DeriveValueType
                )));

                let args = [const_fn, Some(derives), derive_value_type]
                    .into_iter()
                    .flatten();

                Ok(quote! {
                    #[::nutype::nutype(
                        #(#args),*
                    )]
                    #vis struct #name(#inner);
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(quote!(#(#newtypes)*))
    }

    fn generate_input(&self, input: &Input) -> Result<TokenStream> {
        let vis = &self.input.vis;
        let name = &input.name;
        let fields = input
            .fields
            .iter()
            .map(|name| {
                let struct_field = self
                    .fields
                    .named
                    .iter()
                    .find(|f| *f.ident.as_ref().unwrap() == *name)
                    .ok_or(Error::new(
                        name.span(),
                        "this name found in the struct definition",
                    ))?;

                Ok(struct_field)
            })
            .collect::<Result<Vec<_>>>()?;

        let body = if fields.is_empty() {
            quote!(;)
        } else {
            quote!({ #(#fields),* })
        };

        let field_idents: Vec<_> = fields.iter().map(|f| &f.ident).collect();
        let field_tys: Vec<_> = fields.iter().map(|f| &f.ty).collect();

        let from_one_field = if fields.len() == 1 && let Some(field) = fields.first() {
            let id = &field.ident;
            let ty = &field.ty;
            Some(quote! {
                impl From<#ty> for #name {
                    fn from(#id: #ty) -> Self {
                        Self { #id }
                    }
                }
            })
        } else {
            None
        };

        Ok(quote! {
            #[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
            #vis struct #name #body

            impl #name {
                #[allow(clippy::too_many_arguments)]
                #vis fn new(
                    #(#field_idents : impl ::core::convert::Into<#field_tys>),*
                ) -> Self {
                    Self {
                        #(#field_idents : ::core::convert::Into::into(#field_idents)),*
                    }
                }

                fn apply(self, model: Model) -> Model {
                    Model {
                        #(#field_idents : self.#field_idents,)*
                        ..model
                    }
                }

                fn apply_ref(self, model: &mut Model) {
                    #(model.#field_idents = self.#field_idents;)*
                }
            }

            impl From<Model> for #name {
                fn from(model: Model) -> Self {
                    Self {
                        #(#field_idents : model.#field_idents),*
                    }
                }
            }

            #from_one_field
        })
    }

    fn generate_inputs(&self) -> Result<TokenStream> {
        let inputs = self
            .args
            .inputs
            .iter()
            .map(|input| self.generate_input(input))
            .collect::<Result<Vec<_>>>()?;

        Ok(quote!(#(#inputs)*))
    }

    fn generate_impl(&self) -> Result<TokenStream> {
        let vis = &self.input.vis;
        let field_idents: Vec<_> = self.fields.named.iter().map(|f| &f.ident).collect();
        let field_tys: Vec<_> = self.fields.named.iter().map(|f| &f.ty).collect();

        Ok(quote! {
            impl Model {
                #[allow(clippy::too_many_arguments)]
                #vis fn new(
                    #(#field_idents : impl ::core::convert::Into<#field_tys>),*
                ) -> Self {
                    Self {
                        #(#field_idents : ::core::convert::Into::into(#field_idents)),*
                    }
                }
            }
        })
    }

    pub fn generate(self) -> Result<TokenStream> {
        self.check_struct_name()?;

        let newtypes = self.generate_newtypes()?;
        let inputs = self.generate_inputs()?;
        let struct_impl = self.generate_impl()?;

        Ok(quote! {
            #newtypes
            #inputs
            #struct_impl
        })
    }
}
