use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::Token;

fn find_duplicates<'a>(iter: impl Iterator<Item = &'a Ident>) -> HashMap<&'a Ident, usize> {
    let mut seen = HashMap::<&Ident, usize>::new();
    for ident in iter {
        seen.entry(ident).and_modify(|e| *e += 1).or_insert(0);
    }
    seen.retain(|_, v| *v != 0);
    seen
}

pub struct Ripsy {
    pub idents: Punctuated<Ident, Token![,]>,
}

impl Parse for Ripsy {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let idents = input.parse_terminated(Ident::parse, Token![,])?;
        let duplicates = find_duplicates(idents.iter().by_ref());
        if !duplicates.is_empty() {
            let (tokens, err) = duplicates.iter().fold(
                (
                    TokenStream::new(),
                    "duplicate endpoints found: ".to_string(),
                ),
                |(mut ts, mut msg), (ident, _)| {
                    msg.push_str(&format!("`{ident}`, "));
                    ts.extend(quote_spanned!(ident.span()=> #ident));
                    (ts, msg)
                },
            );
            let err = err.strip_suffix(", ").unwrap_or(&err);
            return Err(syn::Error::new_spanned(tokens, err));
        }

        Ok(Self { idents })
    }
}

impl ToTokens for Ripsy {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let routes = self.idents.iter().map(|ident| {
            let path = format!("/{ident}");
            quote_spanned!(ident.span()=> .route(#path, #ident()))
        });

        tokens.extend(quote! {{
            let app = ::ripsy::__macro_helpers::axum::Router::new()
                #(#routes)*;
            app
        }});
    }
}

mod tests {
    #[test]
    fn dups() {
        let a = syn::parse_str("a").unwrap();
        let b = syn::parse_str("b").unwrap();
        let c = syn::parse_str("c").unwrap();
        let dups = super::find_duplicates(vec![&a, &b, &a, &c, &b, &a].into_iter());

        assert_eq!(dups.get(&a), Some(&3));
        assert_eq!(dups.get(&b), Some(&2));
        assert_eq!(dups.get(&c), None);
    }
}
