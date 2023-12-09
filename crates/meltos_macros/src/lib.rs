use proc_macro::TokenStream;

mod display;


#[proc_macro_derive(Display)]
pub fn derive_display(token: TokenStream) -> TokenStream {
    display::display(token)
}
