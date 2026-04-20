use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, Token, Type, parse_macro_input, punctuated::Punctuated};

mod attribute_macros;
mod derive_macros;

/// Derive a pass-through [`Handler`] implementation that forwards every request
/// to `self.next`.
///
/// The struct must have at least one type parameter `T` (the request type) and
/// a field named `next` whose type implements `Handler<T>`.
///
/// # Example
///
/// ```ignore
/// #[derive(Handler)]
/// struct Passthrough<T, N: Handler<T>> {
///     next: N,
///     _phantom: std::marker::PhantomData<T>,
/// }
/// ```
#[proc_macro_derive(Handler)]
pub fn handler_derive(input: TokenStream) -> TokenStream {
    derive_macros::handler_derive_impl(input)
}

struct HandlerList {
    handlers: Vec<Type>,
}

impl syn::parse::Parse for HandlerList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let list = Punctuated::<Type, Token![,]>::parse_terminated(input)?;

        Ok(HandlerList {
            handlers: list.into_iter().collect(),
        })
    }
}

/// Compose handler types into a nested chain.
///
/// Each argument is a handler type whose `new(next)` constructor takes the
/// next handler as its final argument. The macro nests them right-to-left,
/// terminating with the local variable `base_handler`, which must be in
/// scope at the call site.
///
/// # Example
///
/// ```ignore
/// use cor::{Handler, NilHandler, chain, handler};
///
/// #[handler]
/// struct Echo<T> {}
///
/// impl<N: Handler<String>> Handler<String> for Echo<String, N> {
///     fn handle(&self, request: String) {
///         println!("{}", request);
///     }
/// }
///
/// let base_handler = NilHandler::new();
/// let chain = chain![Echo];
/// chain.handle("hello".to_string());
/// ```
#[proc_macro]
pub fn chain(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as HandlerList);
    let handlers = input.handlers;

    let expanded = if handlers.is_empty() {
        quote! { base_handler }
    } else {
        handlers
            .iter()
            .rev()
            .fold(quote! { ::cor::NilHandler::new() }, |acc, handler| {
                quote! {
                    #handler::new(#acc)
                }
            })
    };

    TokenStream::from(expanded)
}

struct AppendInput {
    new_handlers: Vec<Type>,
    existing_chain: Expr,
}

impl syn::parse::Parse for AppendInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let handlers = Punctuated::<Type, Token![,]>::parse_separated_nonempty(input)?;

        input.parse::<Token![;]>()?;

        let chain = input.parse::<Expr>()?;

        Ok(AppendInput {
            new_handlers: handlers.into_iter().collect(),
            existing_chain: chain,
        })
    }
}

#[proc_macro]
pub fn append_chain(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AppendInput);

    let existing_chain = &input.existing_chain;
    let handlers = &input.new_handlers;

    let expanded = if handlers.is_empty() {
        quote! { #existing_chain }
    } else {
        handlers
            .iter()
            .fold(quote! { #existing_chain }, |acc, handler| {
                quote! {
                    #acc.append(#handler::new(::cor::NilHandler))
                }
            })
    };

    TokenStream::from(expanded)
}

/// Generate the scaffolding for a chain-of-responsibility handler struct.
///
/// Apply this attribute to a struct with a single type parameter `T`. The macro
/// generates:
///
/// - An additional generic `N: Handler<T>` for the next handler
/// - A `next: N` field and a `_phantom: PhantomData<T>` field
/// - A `new(<user fields>, next)` constructor
///
/// Any user-defined fields are preserved and appear as positional arguments of
/// `new` in declaration order, followed by `next`. You then write your own
/// `Handler<T>` implementation to define the routing logic.
///
/// # Example
///
/// ```ignore
/// use cor::{Handler, handler};
///
/// #[handler]
/// struct Logger<T> {}
///
/// impl<N: Handler<String>> Handler<String> for Logger<String, N> {
///     fn handle(&self, request: String) {
///         if !request.is_empty() {
///             println!("{}", request);
///         } else {
///             self.next.handle(request);
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    attribute_macros::handler_attr_impl(attr, item)
}
