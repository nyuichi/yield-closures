//! # yield-closures
//!
//! An implementation of [MCP-49](https://github.com/rust-lang/lang-team/issues/49).
//!
//! ```rust
//! #[test]
//! fn decode_escape_string() {
//!     let escaped_text = "Hello,\x20world!\\n";
//!     let text: String = escaped_text
//!         .chars()
//!         .filter_map(co!(|c| {
//!             loop {
//!                 if c != '\\' {
//!                     Not escaped
//!                     yield Some(c);
//!                     continue;
//!                 }
//!
//!                 Go past the \
//!                 yield None;
//!
//!                 Unescaped-char
//!                 match c {
//!                     Hexadecimal
//!                     'x' => {
//!                         yield None; Go past the x
//!                         let most = c.to_digit(16);
//!                         yield None; Go past the first digit
//!                         let least = c.to_digit(16);
//!                         Yield the decoded char if valid
//!                         yield (|| char::from_u32(most? << 4 | least?))()
//!                     }
//!                     Simple escapes
//!                     'n' => yield Some('\n'),
//!                     'r' => yield Some('\r'),
//!                     't' => yield Some('\t'),
//!                     '0' => yield Some('\0'),
//!                     '\\' => yield Some('\\'),
//!                     Unnecessary escape
//!                     _ => yield Some(c),
//!                 }
//!             }
//!         }))
//!         .collect();
//!     assert_eq!(text, "Hello, world!\n");
//! }
//! ```
//!
//! For the details of the proposal, see https://lang-team.rust-lang.org/design_notes/general_coroutines.html.
//!
//! Differences between this implementation and the proposal are summarized below:
//!
//! - This crate offers a macro implementation. It works with the stable Rust.
//! - No `FnPin` is provided. Yield closures made with this crate use `Box::pin` internally and hence `FnMut`.
//! - In yield closures, one cannot use `return` expressions.
//! - The body of a yield closure must be explosive i.e. must not return and typed by the `!` type. Thus it is compatible with both of the two designs of yield closures discussed in the document of MCP-49: poisoning by default or not.

use proc_macro::*;
use syn::{fold::Fold, ReturnType};

#[proc_macro]
pub fn co(input: TokenStream) -> TokenStream {
    let closure: syn::ExprClosure = syn::parse2(input.into()).unwrap();

    if !closure.attrs.is_empty() {
        unimplemented!("attributes");
    }
    if closure.asyncness.is_some() {
        unimplemented!("async closure");
    }
    if closure.movability.is_some() {
        unimplemented!("movability");
    }

    let mut types: Vec<Option<syn::Type>> = vec![];
    let inputs = closure
        .inputs
        .iter()
        .map(|input| match input {
            syn::Pat::Ident(ident) => {
                if !ident.attrs.is_empty() {
                    unimplemented!("attributes");
                }
                if ident.subpat.is_some() {
                    unimplemented!("subpatterns");
                }
                if ident.by_ref.is_some() {
                    unimplemented!("reference patterns");
                }
                if ident.mutability.is_some() {
                    unimplemented!("mutable parameter");
                }
                types.push(None);
                ident.ident.clone()
            }
            syn::Pat::Type(syn::PatType {
                attrs,
                pat,
                colon_token: _colon_token,
                ty,
            }) => {
                if !attrs.is_empty() {
                    unimplemented!("attributes");
                }
                match &**pat {
                    syn::Pat::Ident(ident) => {
                        if !ident.attrs.is_empty() {
                            unimplemented!("attributes");
                        }
                        if ident.subpat.is_some() {
                            unimplemented!("subpatterns");
                        }
                        if ident.by_ref.is_some() {
                            unimplemented!("reference patterns");
                        }
                        if ident.mutability.is_some() {
                            unimplemented!("mutable parameter");
                        }
                        types.push(Some((**ty).clone()));
                        ident.ident.clone()
                    }
                    _ => {
                        unimplemented!("patterns inside type patterns must be variable");
                    }
                }
            }
            p => {
                unimplemented!("input pattern must be variable: {:?}", p);
            }
        })
        .collect::<Vec<_>>();

    let ret_ty;
    match &closure.output {
        ReturnType::Default => {
            ret_ty = None;
        }
        ReturnType::Type(_, ty) => {
            ret_ty = Some(ty.clone());
        }
    }

    let body = ReplaceYields { inputs: &inputs }.fold_expr(*closure.body);

    let capture = closure.capture;

    let co_imp_fn = match inputs.len() {
        0 => {
            let ret_ty = ret_ty
                .map(|t| quote::quote!(#t))
                .unwrap_or(quote::quote!(_));
            quote::quote!(::yield_closures::co0::<_, #ret_ty, _>)
        }
        1 => {
            let ret_ty = ret_ty
                .map(|t| quote::quote!(#t))
                .unwrap_or(quote::quote!(_));
            let a0 = types[0]
                .as_ref()
                .map(|t| quote::quote!(#t))
                .unwrap_or(quote::quote!(_));
            quote::quote!(::yield_closures::co::<_, #a0, #ret_ty, _>)
        }
        2 => {
            let ret_ty = ret_ty
                .map(|t| quote::quote!(#t))
                .unwrap_or(quote::quote!(_));
            let a0 = types[0]
                .as_ref()
                .map(|t| quote::quote!(#t))
                .unwrap_or(quote::quote!(_));
            let a1 = types[1]
                .as_ref()
                .map(|t| quote::quote!(#t))
                .unwrap_or(quote::quote!(_));
            quote::quote!(::yield_closures::co2::<_, #a0, #a1, #ret_ty, _>)
        }
        3 => {
            let ret_ty = ret_ty
                .map(|t| quote::quote!(#t))
                .unwrap_or(quote::quote!(_));
            let a0 = types[0]
                .as_ref()
                .map(|t| quote::quote!(#t))
                .unwrap_or(quote::quote!(_));
            let a1 = types[1]
                .as_ref()
                .map(|t| quote::quote!(#t))
                .unwrap_or(quote::quote!(_));
            let a2 = types[1]
                .as_ref()
                .map(|t| quote::quote!(#t))
                .unwrap_or(quote::quote!(_));
            quote::quote!(::yield_closures::co3::<_, #a0, #a1, #a2, #ret_ty, _>)
        }
        _ => {
            unimplemented!("the number of inputs is too large");
        }
    };
    quote::quote!(
        #co_imp_fn(|__arg_rx, __yield_tx| async #capture {
            let (__arg_rx, __yield_tx) = async move { (__arg_rx, __yield_tx) }.await; do partial move
            let (#( mut #inputs ),*);
            ::yield_closures::reassign_args!(__arg_rx, #( #inputs, )*);
            #body
        })
    )
    .into()
}

struct ReplaceYields<'a> {
    inputs: &'a [syn::Ident],
}

impl<'a> Fold for ReplaceYields<'a> {
    fn fold_expr(&mut self, i: syn::Expr) -> syn::Expr {
        match i {
            syn::Expr::Yield(i) => {
                let expr = if let Some(expr) = i.expr {
                    self.fold_expr(*expr)
                } else {
                    syn::parse2(quote::quote!(())).unwrap()
                };
                let inputs = self.inputs;
                syn::parse2(quote::quote! {{
                    __yield_tx.send(#expr).unwrap();
                    ::yield_closures::drop_args!(#( #inputs, )*);
                    ::yield_closures::pend_once().await;
                    ::yield_closures::reassign_args!(__arg_rx, #( #inputs, )*);
                }})
                .unwrap()
            }
            syn::Expr::Await(_) => {
                unimplemented!("await expressions in yield closures");
            }
            syn::Expr::Async(_) => {
                unimplemented!("async blocks in yield closures");
            }
            syn::Expr::TryBlock(_) => {
                unimplemented!("try blocks in yield closures");
            }
            syn::Expr::Return(_) => {
                panic!("return expressions in yield closures are unsupported");
            }
            _ => syn::fold::fold_expr(self, i),
        }
    }
}
