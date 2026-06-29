use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream};
use syn::{Result, Token, Type};

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
pub struct Input {
    pub name: Ident,
    pub fields: Vec<Ident>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let content;
        syn::parenthesized!(content in input);

        let fields = content
            .parse_terminated(Ident::parse, Token![,])?
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
