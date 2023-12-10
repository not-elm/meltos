use proc_macro::TokenStream;

use syn::__private::TokenStream2;

mod deref;
mod display;


#[proc_macro_derive(Display)]
pub fn derive_display(token: TokenStream) -> TokenStream {
    display::display(token)
}


#[proc_macro_derive(Deref)]
pub fn derive_deref(token: TokenStream) -> TokenStream {
    to_plain_token(deref::deref(token))
}


fn to_plain_token(result: syn::Result<TokenStream2>) -> TokenStream {
    result.unwrap_or_else(|e| e.to_compile_error()).into()
}
