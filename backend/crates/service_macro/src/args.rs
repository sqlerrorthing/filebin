use proc_macro2::Span;
use syn::{Error, Token, Result};
use syn::parse::{Parse, ParseStream};
use crate::kw;

#[derive(Debug, Copy, Clone)]
pub struct Args {
    pub require_send: bool,
    pub require_sync: bool
}

fn try_parse(input: ParseStream) -> Result<Args> {
    let mut require_send = true;
    let mut require_sync = true;
    if input.is_empty() {
        return Ok(Args { require_send, require_sync });
    }

    while !input.is_empty() {
        if input.peek(Token![?]) {
            input.parse::<Token![?]>()?;
            if input.peek(kw::Send) {
                input.parse::<kw::Send>()?;
                require_send = false;
            } else if input.peek(kw::Sync) {
                input.parse::<kw::Sync>()?;
                require_sync = false;
            }
        }

        if input.peek(Token![,]) { input.parse::<Token![,]>()?; }
    }

    Ok(Args { require_send, require_sync })
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        match try_parse(input) {
            Ok(args) if input.is_empty() => Ok(args),
            _ => Err(Error::new(Span::call_site(), "expected #[service] or #[service(?Send, ?Sync)] or something"))
        }
    }
}