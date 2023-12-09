use proc_macro::TokenStream;

use syn::ItemStruct;
use syn::__private::quote::quote;
use syn::__private::TokenStream2;


pub fn display(token: TokenStream) -> TokenStream {
    impl_display(token)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}


fn impl_display(token: TokenStream) -> syn::Result<TokenStream2> {
    let item = syn::parse::<ItemStruct>(token)?;
    let ident = item.ident;
    Ok(quote! {
        impl std::fmt::Display for #ident{
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    })
}
