use crate::args::{Args, Input, InputField};
use crate::newtype::NewtypeMeta;
use derive_new::new;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{DeriveInput, Error, Field, FieldsNamed, Result, parse_quote};

#[derive(new)]
pub struct Generator {
    args: Args,
    input: DeriveInput,
    fields: FieldsNamed,
    _root: TokenStream,
}

#[derive(new)]
struct ResolvedInputField<'a> {
    name: &'a Ident,
    struct_name: &'a Ident,
    struct_field: Field,
    spread: bool,
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
        let resolved_fields = input
            .fields
            .iter()
            .map(|input_field| {
                let (name, alias, struct_field, spread) = match input_field {
                    InputField::Field { name, alias } => {
                        let mut struct_field = self
                            .fields
                            .named
                            .iter()
                            .find(|f| *f.ident.as_ref().unwrap() == *name)
                            .ok_or(Error::new(
                                name.span(),
                                "this name found in the struct definition",
                            ))?
                            .clone();

                        struct_field.ident = Some(alias.clone());
                        (name, alias, struct_field, false)
                    }
                    InputField::Spread { name, path } => (
                        name,
                        name,
                        parse_quote! {
                            #vis #name: #path
                        },
                        true,
                    ),
                };

                Ok(ResolvedInputField::new(name, alias, struct_field, spread))
            })
            .collect::<Result<Vec<_>>>()?;

        let body = if resolved_fields.is_empty() {
            quote!(;)
        } else {
            let struct_fields = resolved_fields.iter().map(|f| &f.struct_field);
            quote!({ #(#struct_fields),* })
        };

        let struct_field_idents: Vec<_> = resolved_fields.iter().map(|f| &f.struct_name).collect();
        let field_idents: Vec<_> = resolved_fields.iter().map(|f| &f.name).collect();
        let field_tys: Vec<_> = resolved_fields.iter().map(|f| &f.struct_field.ty).collect();
        let no_spreads: bool = resolved_fields.iter().all(|f| !f.spread);

        let from_one_field = if resolved_fields.len() == 1 && let Some(field) = resolved_fields.first() {
            let struct_id = &field.struct_field;
            let ty = &field.struct_field.ty;
            Some(quote! {
                impl From<#ty> for #name {
                    fn from(#struct_id: #ty) -> Self {
                        Self {
                            #struct_id
                        }
                    }
                }
            })
        } else {
            None
        };

        let model_related_methods = no_spreads.then(|| {
            quote! {
                impl #name {
                    fn apply(self, model: Model) -> Model {
                        Model {
                            #(#field_idents : self.#struct_field_idents,)*
                            ..model
                        }
                    }

                    fn apply_ref(self, model: &mut Model) {
                        #(model.#field_idents = self.#struct_field_idents;)*
                    }
                }

                impl From<Model> for #name {
                    fn from(model: Model) -> Self {
                        Self {
                            #(#struct_field_idents : model.#field_idents),*
                        }
                    }
                }
            }
        });

        Ok(quote! {
            #[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
            #vis struct #name #body

            impl #name {
                #[allow(clippy::too_many_arguments)]
                #vis fn new(
                    #(#struct_field_idents : impl ::core::convert::Into<#field_tys>),*
                ) -> Self {
                    Self {
                        #(#struct_field_idents : ::core::convert::Into::into(#struct_field_idents)),*
                    }
                }
            }
            
            #model_related_methods
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
