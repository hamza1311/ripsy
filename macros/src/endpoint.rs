use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_quote, ItemFn, ReturnType, Signature, Type};

use crate::macros::ensure;
use crate::Method;

pub struct Endpoint {
    ident: Ident,
    inputs: Vec<syn::PatType>,
    output: Box<Type>,
    vis: syn::Visibility,
    block: Box<syn::Block>,
}

impl Parse for Endpoint {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item = input.parse::<ItemFn>()?;

        let Signature {
            constness,
            asyncness,
            abi,
            generics,
            ..
        } = &item.sig;
        ensure!(
            constness.is_none(),
            "endpoint must not be const",
            constness.span()
        );
        ensure!(
            asyncness.is_some(),
            "endpoint must be async",
            asyncness.span()
        );
        ensure!(
            abi.is_none(),
            "endpoint must not be an extern fn",
            abi.span()
        );
        ensure!(
            generics.params.is_empty(),
            "endpoint must not have generics",
            generics.span()
        );
        ensure!(
            item.sig.receiver().is_none(),
            "endpoint must not have a receiver",
            item.sig.receiver().span()
        );

        let output = match item.sig.output {
            ReturnType::Default => parse_quote!(()),
            ReturnType::Type(_, ty) => ty,
        };

        Ok(Self {
            ident: item.sig.ident,
            inputs: item
                .sig
                .inputs
                .into_iter()
                .map(|arg| match arg {
                    syn::FnArg::Typed(arg) => Ok(arg),
                    _ => Err(syn::Error::new(
                        arg.span(),
                        "endpoint arguments must be named",
                    )),
                })
                .collect::<syn::Result<_>>()?,
            output,
            vis: item.vis,
            block: item.block,
        })
    }
}

impl Endpoint {
    fn args(&self) -> syn::PatType {
        // Json((...inputs,)): Json((...inputs_type,))
        let inputs = self.inputs.iter().map(|arg| &arg.pat);
        let inputs_type = self.inputs.iter().map(|arg| &arg.ty);

        let pat = parse_quote!(::ripsy::Bincode((#(#inputs,)*)));
        let ty = parse_quote!(::ripsy::Bincode<(#(#inputs_type,)*)>);
        syn::PatType {
            attrs: vec![],
            pat: Box::new(pat),
            colon_token: Default::default(),
            ty: Box::new(ty),
        }
    }

    fn to_server_tokens(&self, method: Method) -> TokenStream {
        let Self {
            vis,
            ident,
            block,
            inputs,
            output,
            ..
        } = self;
        let args = self.args();
        let inner_args = self.inputs.iter().map(|arg| &arg.pat);

        let handler = quote! {
            #vis async fn handler(#args) -> ::ripsy::Bincode<#output> {
                async fn inner(#(#inputs,)*) -> #output {
                    #block
                }
                let output = inner(#(#inner_args,)*).await;
                ::ripsy::Bincode(output)
            }
        };

        let method_router = match method {
            Method::Mutation => quote!(post),
            Method::Query => quote!(get),
        };

        let http_body = quote!(::ripsy::__macro_helpers::axum::body::HttpBody);
        let send = quote!(::std::marker::Send);
        let sync = quote!(::std::marker::Sync);
        quote! {
            #vis fn #ident<S, B>() -> ::ripsy::__macro_helpers::axum::routing::MethodRouter<S, B>
            where
                B: #http_body + Send + 'static,
                <B as #http_body>::Error: #send + #sync + ::std::error::Error + 'static,
                <B as #http_body>::Data: #send + #sync + 'static,
                S: ::std::clone::Clone + #send + #sync + 'static,
            {
                #handler

                ::ripsy::__macro_helpers::axum::routing::#method_router(handler)
            }
        }
    }

    fn to_client_tokens(&self, method: Method) -> TokenStream {
        let Self {
            vis,
            ident,
            inputs,
            output,
            ..
        } = self;

        let endpoint_type = match method {
            Method::Mutation => quote!(::ripsy::EndpointType::Mutation),
            Method::Query => quote!(::ripsy::EndpointType::Query),
        };
        let inner_args = self.inputs.iter().map(|arg| &arg.pat);

        quote! {
            #vis async fn #ident(#(#inputs,)*) -> Result<#output, ()> {
                let body = ::ripsy::Bincode(#(#inner_args,)*);
                let body = body.serialize().unwrap();
                ::ripsy::client::request(stringify!(#ident), body, #endpoint_type).await
            }
        }
    }

    pub(crate) fn to_tokens(&self, method: Method) -> TokenStream {
        let mut tokens = TokenStream::new();
        if cfg!(feature = "server") {
            tokens.extend(self.to_server_tokens(method));
        } else if cfg!(feature = "client") {
            tokens.extend(self.to_client_tokens(method));
        }
        tokens
    }
}
