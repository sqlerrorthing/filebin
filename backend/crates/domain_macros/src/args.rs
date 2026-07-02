use heck::ToSnakeCase;
use proc_macro2::Ident;
use quote::format_ident;
use syn::parse::{Parse, ParseStream};
use syn::{Result, Token, Type};
use syn::token::Paren;

#[derive(Debug, Clone)]
pub struct NewType {
    pub name: Ident,
    pub inner: Type,
}

impl Parse for NewType {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let content;
        syn::parenthesized!(content in input);
        let inner: Type = content.parse()?;
        Ok(NewType { name, inner })
    }
}

#[derive(Debug, Clone)]
pub enum InputField {
    Field {
        name: Ident,
        alias: Ident
    },
    Spread {
        name: Ident,
        path: syn::Path,
    },
}

impl Parse for InputField {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![..]) {
            input.parse::<Token![..]>()?;
            let path: syn::Path = input.parse()?;

            let prefix = if input.peek(Token![as]) {
                input.parse::<Token![as]>()?;
                input.parse::<Ident>()?
            } else {
                format_ident!("{}", path.segments.last().unwrap().ident.to_string().to_snake_case())
            };

            return Ok(InputField::Spread { path, name: prefix });
        }

        let name: Ident = input.parse()?;
        let alias = if input.peek(Token![as]) {
            input.parse::<Token![as]>()?;
            input.parse()?
        } else {
            name.clone()
        };

        Ok(InputField::Field { name, alias })
    }
}

#[derive(Debug, Clone)]
pub struct Input {
    pub name: Ident,
    pub fields: Vec<InputField>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;

        if !input.peek(Paren) {
            return Ok(Input { name, fields: vec![] })
        }

        let content;
        syn::parenthesized!(content in input);

        let fields = content
            .parse_terminated(InputField::parse, Token![,])?
            .into_iter()
            .collect();

        Ok(Input { name, fields })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Args {
    pub newtypes: Vec<NewType>,
    pub inputs: Vec<Input>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = Args::default();

        while !input.is_empty() {
            let kw: Ident = input.parse()?;
            let content;
            syn::parenthesized!(content in input);

            if kw == "newtypes" {
                let parsed_newtypes = content.parse_terminated(NewType::parse, Token![,])?;
                args.newtypes = parsed_newtypes.into_iter().collect();
            } else if kw == "inputs" {
                let parsed_inputs = content.parse_terminated(Input::parse, Token![,])?;
                args.inputs = parsed_inputs.into_iter().collect();
            } else {
                return Err(input.error("expected 'newtypes' or 'inputs'"));
            }

            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
        }

        Ok(args)
    }
}
