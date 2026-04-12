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

/// Generate a complete handler struct with condition-based routing.
///
/// Apply this attribute to a struct with a single type parameter `T`. The macro
/// generates:
///
/// - An additional generic `N: Handler<T>` for the next handler
/// - Fields: `next: N`, `condition: Box<dyn Fn(&T) -> bool>`, `on_match: Box<dyn Fn(&T)>`
/// - A `new(condition, on_match, next)` constructor
/// - A `Handler<T>` implementation that calls `on_match` when `condition` returns
///   `true`, otherwise forwards to `next`
///
/// Any user-defined fields (other than `next`) are preserved.
///
/// # Example
///
/// ```ignore
/// #[handler]
/// struct Logger<T> {}
///
/// let chain = chain![
///     |next| Logger::new(|req: &String| !req.is_empty(), |req| println!("{}", req), next),
/// ];
/// ```
#[proc_macro_attribute]
pub fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    attribute_macros::handler_attr_impl(attr, item)
}
