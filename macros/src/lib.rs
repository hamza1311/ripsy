mod endpoint;
mod macros;
mod ripsy;

use proc_macro2::Ident;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Method {
    Mutation,
    Query,
}

impl Parse for Method {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse().map_err(|e| {
            syn::Error::new(
                e.span(),
                "unexpected end of input, expected `mutation` or `query`",
            )
        })?;

        match &*ident.to_string() {
            "mutation" => Ok(Method::Mutation),
            "query" => Ok(Method::Query),
            actual => Err(syn::Error::new(
                ident.span(),
                format!("expected `mutation` or `query`, found {}", actual),
            )),
        }
    }
}

#[proc_macro]
pub fn ripsy(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(input as ripsy::Ripsy);
    item.into_token_stream().into()
}

#[proc_macro_attribute]
pub fn endpoint(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = syn::parse_macro_input!(attr as Method);
    let item = syn::parse_macro_input!(item as endpoint::Endpoint);

    item.to_tokens(attr).into()
}
