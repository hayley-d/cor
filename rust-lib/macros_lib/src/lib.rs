use proc_macro::TokenStream;

mod attribute_macros;
mod derive_macros;

#[proc_macro_derive(Handler)]
pub fn handler_derive(input: TokenStream) -> TokenStream {
    derive_macros::handler_derive_impl(input)
}

#[proc_macro_attribute]
pub fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    attribute_macros::handler_attr_impl(attr, item)
}
