use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

pub fn handler_attr_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::DeriveInput);

    let name = &input.ident;
    let visibility = &input.vis;
    let attributes = &input.attrs;
    let generics = &input.generics;

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

    // Filter out `next` field if user included it (macro manages it)
    let existing_field_tokens: Vec<_> = existing_fields
        .iter()
        .filter(|f| f.ident.as_ref().map_or(true, |id| id != "next"))
        .map(|f| quote! { #f })
        .collect();

    let expanded = quote! {
        #(#attributes)*
        #visibility struct #name<#type_t, N: ::cor::Handler<#type_t>> {
            #(#existing_field_tokens,)*
            next: N,
            condition: Box<dyn Fn(&#type_t) -> bool>,
            on_match: Box<dyn Fn(&#type_t)>,
        }

        impl<#type_t, N: ::cor::Handler<#type_t>> #name<#type_t, N> {
            pub fn new(
                condition: impl Fn(&#type_t) -> bool + 'static,
                on_match: impl Fn(&#type_t) + 'static,
                next: N,
            ) -> Self {
                Self {
                    next,
                    condition: Box::new(condition),
                    on_match: Box::new(on_match),
                }
            }
        }

        impl<#type_t, N: ::cor::Handler<#type_t>> ::cor::Handler<#type_t> for #name<#type_t, N> {
            fn handle(&self, request: #type_t) {
                if (self.condition)(&request) {
                    (self.on_match)(&request);
                } else {
                    self.next.handle(request);
                }
            }
        }
    };

    expanded.into()
}
