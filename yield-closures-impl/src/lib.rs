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
            let (__arg_rx, __yield_tx) = async move { (__arg_rx, __yield_tx) }.await; // do partial move
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
