use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

pub fn handler_derive_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;

    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let type_param = generics
        .type_params()
        .next()
        .expect("Expected at least one generic type parameter");

    let t_param_name = &type_param.ident;

    let expanded = quote! {
        impl #impl_generics ::cor::Handler<#t_param_name> for #name #ty_generics #where_clause {
            fn handle(&self, request: #t_param_name) {
                self.next.handle(request);
            }
        }
    };

    expanded.into()
}
