use crate::TokenResult;
use proc_macro::TokenStream;
use syn::ItemStruct;
use syn::__private::quote::quote;

pub fn sha1(token: TokenStream) -> TokenResult {
    let item = syn::parse::<ItemStruct>(token)?;
    let name = item.ident;
    Ok(quote! {
        impl #name{
            #[inline(always)]
            pub fn new() -> Self{
                Self(meltos_util::hash::random())
            }
        }

        impl Default for #name{
            #[inline(always)]
            fn default() -> Self{
                Self::new()
            }
        }
    })
}
