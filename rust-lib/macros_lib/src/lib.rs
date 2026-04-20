use proc_macro::TokenStream;

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
