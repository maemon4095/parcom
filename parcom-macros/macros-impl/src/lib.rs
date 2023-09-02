use proc_macro2::TokenStream;
use quote::quote;

pub fn parser(attr: TokenStream, body: TokenStream) -> TokenStream {
    let item_fn: syn::ItemFn = match syn::parse2(body) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error(),
    };

    let sig = item_fn.sig;
    quote! {}
}
