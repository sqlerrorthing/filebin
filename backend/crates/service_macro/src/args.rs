use proc_macro2::TokenStream;
use std::fmt::Display;
use bitflags::bitflags;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Ident, Result, Token, parenthesized, Lifetime};

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct Requires: u8 {
        const SEND = 1;
        const SYNC = 1 << 1;
        const DEBUG = 1 << 2;
        const STATIC = 1 << 3;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AttrArgs {
    pub requires: Requires,
    pub dynamic_dispatch: bool
}

#[derive(Debug, Clone)]
pub struct Args {
    pub requires: Requires,
    pub dynamic_dispatch: bool,
    pub service_crate_root: TokenStream
}

enum RequiredItem {
    Ident(Ident),
    Lifetime(Lifetime),
}

impl Display for RequiredItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            RequiredItem::Ident(s) => s.to_string(),
            RequiredItem::Lifetime(s) => s.to_string()
        };
        write!(f, "{}", str)
    }
}

impl Parse for RequiredItem {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Ok(lt) = input.parse::<Lifetime>() {
            Ok(RequiredItem::Lifetime(lt))
        } else {
            let id: Ident = input.parse()?;
            Ok(RequiredItem::Ident(id))
        }
    }
}

fn try_parse_required(input: Punctuated<RequiredItem, Token![,]>, requires: &mut Requires) -> Result<()> {
    let mut to_remove = Requires::empty();

    for ident in input {
        match ident.to_string().as_str() {
            "Send" => to_remove |= Requires::SEND,
            "Sync" => to_remove |= Requires::SYNC,
            "Debug" => to_remove |= Requires::DEBUG,
            "'static" => to_remove |= Requires::STATIC,
            _ => {}
        }
    }

    requires.remove(to_remove);
    Ok(())
}

impl Parse for AttrArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut requires = Requires::all();
        let mut dynamic_dispatch = false;

        while !input.is_empty() {
            if input.peek(Token![?]) {
                let _: Token![?] = input.parse()?;
                let content;
                parenthesized!(content in input);
                try_parse_required(
                    content.parse_terminated(RequiredItem::parse, Token![,])?,
                    &mut requires,
                )?;
            } else if input.peek(Ident) {
                let ident: Ident = input.parse()?;
                if ident == "dynamic" {
                    dynamic_dispatch = true
                }
            }

            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
        }

        Ok(AttrArgs {
            requires,
            dynamic_dispatch,
        })
    }
}
