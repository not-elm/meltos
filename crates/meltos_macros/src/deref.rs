use proc_macro::TokenStream;

use syn::Fields::Unnamed;
use syn::__private::quote::quote;
use syn::__private::TokenStream2;
use syn::{Fields, ItemStruct, Type};

pub fn deref(token: TokenStream) -> syn::Result<TokenStream2> {
    let item = syn::parse::<ItemStruct>(token)?;
    let ident = &item.ident;
    let ty = parse_new_type_ty(item.fields).ok_or(syn::Error::new(
        proc_macro2::Span::call_site(),
        "not new type struct",
    ))?;

    Ok(quote! {
        impl std::ops::Deref for #ident {
            type Target = #ty;
            fn deref(&self) -> & Self::Target {
                &self.0
            }
        }
    })
}


fn parse_new_type_ty(fields: Fields) -> Option<Type> {
    if let Unnamed(un_named) = fields {
        Some(un_named.unnamed.first()?.ty.clone())
    } else {
        None
    }
}
