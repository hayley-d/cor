use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

pub fn handler_attr_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::DeriveInput);

    let name = &input.ident;
    let visibility = &input.vis;
    let attributes = &input.attrs;
    let generics = &input.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let type_parameter = generics
        .type_params()
        .next()
        .expect("Expected at least one generic type parameter");
    let type_t = &type_parameter.ident;

    let existing_fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(named) => &named.named,
            _ => panic!("handler attribute requires named fields"),
        },
        _ => panic!("handler attribute only works on structs"),
    };

    let existing_field_tokens = existing_fields.iter().map(|f| quote! { #f });

    let expanded = quote! {
        #(#attributes)*
        #visibility struct #name #generics #where_clause {
            #(#existing_field_tokens,)*
            condition: Box<dyn Fn(&#type_t) -> bool>,
            on_match: Box<dyn Fn(&#type_t)>,
        }

        impl #impl_generics #name #type_generics #where_clause {
            pub fn new(
                condition: impl Fn(&#type_t) -> bool + 'static,
                on_match: impl Fn(&#type_t) + 'static,
            ) -> Self {
                Self {
                    next: None,
                    condition: Box::new(condition),
                    on_match: Box::new(on_match),
                }
            }
        }

        impl #impl_generics ::cor::Handler<#type_t> for #name #type_generics #where_clause {
            fn set_next(&mut self, next: Box<dyn ::cor::Handler<#type_t>>) {
                self.next = Some(next);
            }

            fn handle(&self, request: #type_t) {
                if (self.condition)(&request) {
                    (self.on_match)(&request);
                } else if let Some(ref next) = self.next {
                    next.handle(request);
                }
            }
        }
    };

    expanded.into()
}
