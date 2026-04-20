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

    let existing_field_tokens: Vec<_> = existing_fields
        .iter()
        .filter(|field| field.ident.as_ref().map_or(true, |id| id != "next"))
        .map(|field| quote! { #field })
        .collect();

    let field_names: Vec<_> = existing_fields
        .iter()
        .filter(|field| field.ident.as_ref().map_or(true, |id| id != "next"))
        .map(|field| &field.ident)
        .collect();

    let expanded = quote! {
            #(#attributes)*
            #visibility struct #name<#type_t, N: ::cor::Handler<#type_t>> {
                #(#existing_field_tokens,)*
                pub next: N,
                _phantom: ::std::marker::PhantomData<#type_t>,
            }

            impl<#type_t, N: ::cor::Handler<#type_t>> #name<#type_t, N> {
                pub fn new(
                    #(#existing_field_tokens,)*
                    next: N,
                ) -> Self {
                    Self {
                        #(#field_names,)*
                        next,
                        _phantom: ::std::marker::PhantomData,
                    }
                }
            }

    impl<#type_t, N> ::cor::Linker<#type_t, N> for #name<#type_t, N>
    where
        N: ::cor::Handler<#type_t> + ::cor::Linker<#type_t, N>,
        <N as ::cor::Linker<#type_t, N>>::Output: ::cor::Handler<#type_t>,
        #name<#type_t, <N as ::cor::Linker<#type_t, N>>::Output>: ::cor::Handler<#type_t>,
    {
        type Output = #name<#type_t, <N as ::cor::Linker<#type_t, N>>::Output>;

        fn append(self, new_handler: N) -> Self::Output {
            #name {
                #(#field_names: self.#field_names,)*
                next: self.next.append(new_handler),
                _phantom: ::std::marker::PhantomData,
            }
        }
    }
        };

    expanded.into()
}
