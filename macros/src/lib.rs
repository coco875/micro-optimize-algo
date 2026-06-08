use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn scenario(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn variant(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
