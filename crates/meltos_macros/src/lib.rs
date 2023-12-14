use proc_macro::TokenStream;

use syn::__private::TokenStream2;

mod deref;
mod display;
mod sha;

#[proc_macro_derive(Display)]
pub fn derive_display(token: TokenStream) -> TokenStream {
    display::display(token)
}

#[proc_macro_derive(Deref)]
pub fn derive_deref(token: TokenStream) -> TokenStream {
    to_plain_token(deref::deref(token))
}

#[proc_macro_derive(DerefMut)]
pub fn derive_deref_mut(token: TokenStream) -> TokenStream {
    to_plain_token(deref::deref_mut(token))
}

#[proc_macro_derive(Sha1)]
pub fn derive_sha1(token: TokenStream) -> TokenStream {
    to_plain_token(sha::sha1(token))
}

fn to_plain_token(result: syn::Result<TokenStream2>) -> TokenStream {
    result.unwrap_or_else(|e| e.to_compile_error()).into()
}

type TokenResult = syn::Result<TokenStream2>;
