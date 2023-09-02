use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    str::FromStr,
};

use proc_macro2::TokenStream;
use quote::quote;
use syn::PathArguments;

fn create_error<M: Display>(msg: M) -> TokenStream {
    TokenStream::from_str(format!("compile_error! ({})", msg).as_str()).unwrap()
}

pub fn parser_transform(attr: TokenStream, body: TokenStream) -> TokenStream {
    let item_fn: syn::ItemFn = match syn::parse2(body) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error(),
    };

    let sig = item_fn.sig;
    let vis = item_fn.vis;
    let body = item_fn.block;
    if sig.inputs.len() != 1 {
        return create_error("functional parser arity must be one.");
    }

    let ident = sig.ident;
    let generics = sig.generics.clone();
    let where_clause = sig.generics.clone().where_clause;

    let phantom_members = sig
        .generics
        .clone()
        .params
        .into_iter()
        .filter_map(|p| match p {
            syn::GenericParam::Lifetime(lt) => {
                let lt = lt.lifetime;
                Some(quote!(&#lt ()))
            }
            syn::GenericParam::Type(t) => {
                let t = t.ident;
                Some(quote!(#t))
            }
            syn::GenericParam::Const(_) => None,
        });

    let input_arg = match sig.inputs.first().unwrap() {
        syn::FnArg::Receiver(_) => unreachable!(),
        syn::FnArg::Typed(p) => p.clone(),
    };
    let input_ty = input_arg.ty.clone();

    let result_ty = match sig.output {
        syn::ReturnType::Default => {
            return create_error("functional parser must specify return type.")
        }

        syn::ReturnType::Type(_, ty) => ty,
    };

    let result_type_error = "functional parser must return result type.";

    // Result型かを判別して，かつ引数を取り出すのは無理がある．別の方法を考えたい．
    let _ = match *result_ty {
        syn::Type::Path(path) => {
            let last = path.path.segments.last().unwrap().clone();
            if last.ident.ne("Result") {
                return create_error(result_type_error);
            }

            let PathArguments::AngleBracketed(args) = last.arguments else {
                return create_error(result_type_error);
            };

            if args.args.len() != 2 {
                return create_error(result_type_error);
            }

            let mut iter = args.args.into_iter();
            let Some(first) = iter.next() else {
                return create_error(result_type_error);
            };
            let Some(last) = iter.next() else {
                return create_error(result_type_error);
            };
        }
        _ => return create_error(result_type_error),
    };

    quote! {
        #vis struct #ident #generics (PhantomData<(#(#phantom_members,)*)>) #where_clause;
        impl #generics ::parcom::Parser<#input_ty> for #ident #generics #where_clause {
            type Output;
            type Error;

            fn parse(&self, #input_arg) -> ::parcom::ParseResult<S, Self> #body
        }
    }
}

struct X<'a, T>(PhantomData<(&'a (), T)>)
where
    T: Debug;

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use proc_macro2::TokenStream;
    use syn::parse;

    use crate::parser_transform;

    #[test]
    fn expand() {
        let body = TokenStream::from_str(
            "pub fn p<'a, S: Debug>(input: S) -> Result<(usize, S), ((), S)> where S: Stream {
                todo!()
            }",
        )
        .unwrap();
        let attr = TokenStream::from_str("").unwrap();

        let transformed = parser_transform(attr, body);

        println!("{}", transformed.to_string())
    }
}
