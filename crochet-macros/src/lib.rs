extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, Ident, ItemFn, Pat, ReturnType, Type};

#[proc_macro_attribute]
pub fn component(_args: TokenStream, input: TokenStream) -> TokenStream {
    let ItemFn { attrs, vis, sig, block } = parse_macro_input!(input as ItemFn);
    let cx = name_of_cx(&sig.inputs).expect("No argument of type `&mut Cx` found");

    // return_ty is explicit to produce better error for return type mismatch.
    let return_ty = return_ty_or_unit(&sig.output);

    let tokens = quote! {
        #[track_caller]
        #(#attrs)*
        #vis #sig {
            ::crochet::Cx::with_loc(
                #cx, ::std::panic::Location::caller(), move |#cx| #return_ty #block
            )
        }
    };
    tokens.into()
}

fn name_of_cx<'a>(args: impl IntoIterator<Item = &'a FnArg>) -> Option<Ident> {
    for arg in args {
        if let FnArg::Typed(arg) = arg {
            if let Type::Reference(ty_ref) = &*arg.ty {
                if let (Some(_), Type::Path(path)) = (ty_ref.mutability, &*ty_ref.elem) {
                    if let Some(seg) = path.path.segments.first() {
                        if seg.ident == "Cx" {
                            if let Pat::Ident(pat_ident) = &*arg.pat {
                                return Some(pat_ident.ident.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn return_ty_or_unit(ty: &ReturnType) -> proc_macro2::TokenStream {
    if ty == &ReturnType::Default {
        quote! { -> () }
    } else {
        quote! { #ty }
    }
}
