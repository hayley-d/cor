//! # cor
//!
//! A compile-time Chain of Responsibility pattern library for Rust.
//!
//! Handlers are composed into statically-typed chains with zero dynamic dispatch
//! overhead in the chain structure. Each handler either processes a request or
//! forwards it to the next handler.
//!
//! # Quick start
//!
//! ```
//! use cor::{Handler, chain, handler};
//!
//! #[handler]
//! struct Greeting<T> {}
//!
//! #[handler]
//! struct Farewell<T> {}
//!
//! let chain = chain![
//!     |next| Greeting::new(
//!         |req: &String| req.starts_with("hello"),
//!         |req| println!("{}", req),
//!         next,
//!     ),
//!     |next| Farewell::new(
//!         |req: &String| req.starts_with("bye"),
//!         |req| println!("{}", req),
//!         next,
//!     ),
//! ];
//!
//! chain.handle("hello world".to_string());
//! ```
//!
//! # How it works
//!
//! The [`chain!`] macro composes handler constructors right-to-left, terminating
//! with a [`NilHandler`] that silently drops unhandled requests. Each entry in
//! the macro is a closure that receives the next handler and returns a configured
//! handler, producing a fully nested type at compile time.

extern crate self as cor;

pub use macros_lib::{Handler, handler};

/// The core trait for all handlers in the chain.
///
/// Implement this trait to define custom handler behavior. For most use cases
/// the [`#[handler]`](macro@handler) attribute macro or [`#[derive(Handler)]`](derive@Handler)
/// will generate the implementation for you.
///
/// # Examples
///
/// Manual implementation:
///
/// ```
/// use cor::Handler;
///
/// struct Upper<N: Handler<String>> {
///     next: N,
/// }
///
/// impl<N: Handler<String>> Handler<String> for Upper<N> {
///     fn handle(&self, request: String) {
///         if request.chars().all(|c| c.is_uppercase()) {
///             println!("ALL CAPS: {}", request);
///         } else {
///             self.next.handle(request);
///         }
///     }
/// }
/// ```
pub trait Handler<T> {
    /// Process a request or forward it to the next handler.
    fn handle(&self, request: T);
}

/// A terminal handler that discards any request it receives.
///
/// `NilHandler` is automatically placed at the end of every chain built by
/// the [`chain!`] macro. You typically don't need to use it directly.
///
/// # Examples
///
/// ```
/// use cor::{Handler, NilHandler};
///
/// let nil = NilHandler::new();
/// nil.handle("this is silently dropped".to_string());
/// ```
pub struct NilHandler;

impl<T> Handler<T> for NilHandler {
    fn handle(&self, _: T) {}
}

/// A pass-through handler that forwards every request to the next handler.
///
/// Useful as a no-op head of a chain or as a base for custom handlers
/// that need unconditional forwarding.
///
/// # Examples
///
/// ```
/// use cor::{Handler, NilHandler, BaseHandler};
///
/// let base: BaseHandler<String, NilHandler> = BaseHandler::new(NilHandler::new());
/// base.handle("forwarded to NilHandler".to_string());
/// ```
#[derive(Handler)]
pub struct BaseHandler<T, N: Handler<T>> {
    pub next: N,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, N: Handler<T>> BaseHandler<T, N> {
    pub fn new(next: N) -> Self {
        BaseHandler {
            next,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl NilHandler {
    pub fn new() -> Self {
        NilHandler {}
    }
}

impl Default for NilHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Compose handlers into a chain.
///
/// Each argument is a closure `|next| -> impl Handler<T>` that receives the
/// next handler in the chain and returns a configured handler. The macro nests
/// them right-to-left, with a [`NilHandler`] at the tail.
///
/// # Examples
///
/// Single handler:
///
/// ```
/// use cor::{Handler, chain, handler};
///
/// #[handler]
/// struct Echo<T> {}
///
/// let c = chain![
///     |next| Echo::new(|_: &String| true, |req| println!("{}", req), next),
/// ];
/// c.handle("test".to_string());
/// ```
///
/// Multiple handlers (evaluated left-to-right):
///
/// ```
/// use cor::{Handler, chain, handler};
///
/// #[handler]
/// struct First<T> {}
///
/// #[handler]
/// struct Second<T> {}
///
/// let c = chain![
///     |next| First::new(|req: &i32| *req == 1, |req| println!("first: {}", req), next),
///     |next| Second::new(|req: &i32| *req == 2, |req| println!("second: {}", req), next),
/// ];
/// c.handle(2);
/// ```
#[macro_export]
macro_rules! chain {
    ($last:expr $(,)?) => {
        $last(::cor::NilHandler::new())
    };

    ($head:expr, $($rest:expr),+ $(,)?) => {{
        $head(chain!($($rest),+))
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[handler]
    struct TestHandler<T> {}

    #[test]
    fn nil_handler_accepts_any_request() {
        let nil = NilHandler::new();
        nil.handle("hello".to_string());
        nil.handle(42);
        nil.handle(vec![1, 2, 3]);
    }

    #[test]
    fn base_handler_forwards_to_next() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let log_clone = log.clone();

        let chain = chain![
            |next| BaseHandler {
                next,
                _phantom: std::marker::PhantomData::<String>,
            },
            |next| TestHandler::new(
                |_: &String| true,
                move |req| log_clone.borrow_mut().push(req.clone()),
                next,
            ),
        ];

        chain.handle("hello".to_string());
        assert_eq!(*log.borrow(), vec!["hello"]);
    }

    #[test]
    fn single_handler_chain_matches() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let log_clone = log.clone();

        let chain = chain![
            |next| TestHandler::new(
                |req: &String| req == "match",
                move |req| log_clone.borrow_mut().push(req.clone()),
                next,
            ),
        ];

        chain.handle("match".to_string());
        assert_eq!(*log.borrow(), vec!["match"]);
    }

    #[test]
    fn single_handler_chain_falls_through() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let log_clone = log.clone();

        let chain = chain![
            |next| TestHandler::new(
                |req: &String| req == "match",
                move |req| log_clone.borrow_mut().push(req.clone()),
                next,
            ),
        ];

        chain.handle("no match".to_string());
        assert!(log.borrow().is_empty());
    }

    #[test]
    fn multi_handler_chain_routes_correctly() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let log_a = log.clone();
        let log_b = log.clone();

        #[handler]
        struct HandlerA<T> {}

        #[handler]
        struct HandlerB<T> {}

        let chain = chain![
            |next| HandlerA::new(
                |req: &String| req == "A",
                move |req| log_a.borrow_mut().push(format!("A:{}", req)),
                next,
            ),
            |next| HandlerB::new(
                |req: &String| req == "B",
                move |req| log_b.borrow_mut().push(format!("B:{}", req)),
                next,
            ),
        ];

        chain.handle("B".to_string());
        chain.handle("A".to_string());
        assert_eq!(*log.borrow(), vec!["B:B", "A:A"]);
    }

    #[test]
    fn unmatched_request_falls_through_entire_chain() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let log_a = log.clone();
        let log_b = log.clone();

        #[handler]
        struct HandlerA<T> {}

        #[handler]
        struct HandlerB<T> {}

        let chain = chain![
            |next| HandlerA::new(
                |req: &String| req == "A",
                move |req| log_a.borrow_mut().push(req.clone()),
                next,
            ),
            |next| HandlerB::new(
                |req: &String| req == "B",
                move |req| log_b.borrow_mut().push(req.clone()),
                next,
            ),
        ];

        chain.handle("C".to_string());
        assert!(log.borrow().is_empty());
    }

    #[test]
    fn first_matching_handler_wins() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let log_first = log.clone();
        let log_second = log.clone();

        #[handler]
        struct First<T> {}

        #[handler]
        struct Second<T> {}

        let chain = chain![
            |next| First::new(
                |_: &String| true,
                move |req| log_first.borrow_mut().push(format!("first:{}", req)),
                next,
            ),
            |next| Second::new(
                |_: &String| true,
                move |req| log_second.borrow_mut().push(format!("second:{}", req)),
                next,
            ),
        ];

        chain.handle("x".to_string());
        assert_eq!(*log.borrow(), vec!["first:x"]);
    }

    #[test]
    fn works_with_non_string_types() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let log_clone = log.clone();

        let chain = chain![
            |next| TestHandler::new(
                |req: &i32| *req > 10,
                move |req| log_clone.borrow_mut().push(*req),
                next,
            ),
        ];

        chain.handle(5);
        assert!(log.borrow().is_empty());

        chain.handle(42);
        assert_eq!(*log.borrow(), vec![42]);
    }
}
