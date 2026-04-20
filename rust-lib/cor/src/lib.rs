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
//! #[derive(Clone)]
//! struct LogRequest {
//!     level: LogLevel,
//!     message: String,
//! }
//!
//! #[derive(Clone, PartialEq)]
//! enum LogLevel {
//!     Info,
//!     Warning,
//!     Error,
//! }
//!
//! #[handler]
//! struct InfoHandler<T> {}
//!
//! impl<N: Handler<LogRequest>> Handler<LogRequest> for InfoHandler<LogRequest, N> {
//!     fn handle(&self, request: LogRequest) {
//!         if request.level == LogLevel::Info {
//!             println!("[INFO] {}", request.message);
//!         } else {
//!             self.next.handle(request);
//!         }
//!     }
//! }
//!
//! #[handler]
//! struct WarningHandler<T> {}
//!
//! impl<N: Handler<LogRequest>> Handler<LogRequest> for WarningHandler<LogRequest, N> {
//!     fn handle(&self, request: LogRequest) {
//!         if request.level == LogLevel::Warning {
//!             println!("[WARN] {}", request.message);
//!         } else {
//!             self.next.handle(request);
//!         }
//!     }
//! }
//!
//! #[handler]
//! struct ErrorHandler<T> {}
//!
//! impl<N: Handler<LogRequest>> Handler<LogRequest> for ErrorHandler<LogRequest, N> {
//!     fn handle(&self, request: LogRequest) {
//!         if request.level == LogLevel::Error {
//!             println!("[ERROR] {}", request.message);
//!         } else {
//!             self.next.handle(request);
//!         }
//!     }
//! }
//!
//! let logger = chain![
//!     |next| InfoHandler::new(next),
//!     |next| WarningHandler::new(next),
//!     |next| ErrorHandler::new(next),
//! ];
//!
//! logger.handle(LogRequest {
//!     level: LogLevel::Info,
//!     message: "Server started on port 8080".into(),
//! });
//! ```
//!
//! # How it works
//!
//! The [`#[handler]`](macro@handler) attribute macro generates a struct with a
//! `next` field and a `new(next)` constructor. You provide the routing logic by
//! writing your own [`Handler<T>`] implementation.
//!
//! The [`chain!`] macro composes handler constructors right-to-left, terminating
//! with a [`NilHandler`] that silently drops unhandled requests. Each entry is a
//! closure that receives the next handler and returns a configured handler,
//! producing a fully nested type at compile time.

extern crate self as cor;

pub use macros_lib::{Handler, handler};

/// The core trait for all handlers in the chain.
///
/// Implement this trait to define custom handler behavior. The
/// [`#[handler]`](macro@handler) attribute macro generates the struct scaffolding
/// (fields and constructor) for you, leaving you to write only the `handle`
/// method. For a pass-through handler, [`#[derive(Handler)]`](derive@Handler)
/// generates the implementation.
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
#[handler]
pub struct BaseHandler<T, N: Handler<T>> {}

impl<T, N: Handler<T>> Handler<T> for BaseHandler<T, N> {
    fn handle(&self, request: T) {
        self.next.handle(request);
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
/// impl<N: Handler<String>> Handler<String> for Echo<String, N> {
///     fn handle(&self, request: String) {
///         println!("{}", request);
///     }
/// }
///
/// let c = chain![
///     |next| Echo::new(next),
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
/// impl<N: Handler<i32>> Handler<i32> for First<i32, N> {
///     fn handle(&self, request: i32) {
///         if request == 1 {
///             println!("first: {}", request);
///         } else {
///             self.next.handle(request);
///         }
///     }
/// }
///
/// #[handler]
/// struct Second<T> {}
///
/// impl<N: Handler<i32>> Handler<i32> for Second<i32, N> {
///     fn handle(&self, request: i32) {
///         if request == 2 {
///             println!("second: {}", request);
///         } else {
///             self.next.handle(request);
///         }
///     }
/// }
///
/// let c = chain![
///     |next| First::new(next),
///     |next| Second::new(next),
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
    struct Recorder<T> {
        log: Rc<RefCell<Vec<String>>>,
        tag: &'static str,
        matches: fn(&String) -> bool,
    }

    impl<N: Handler<String>> Handler<String> for Recorder<String, N> {
        fn handle(&self, request: String) {
            if (self.matches)(&request) {
                self.log
                    .borrow_mut()
                    .push(format!("{}:{}", self.tag, request));
            } else {
                self.next.handle(request);
            }
        }
    }

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

        let chain = chain![|next| BaseHandler::new(next), |next| Recorder::new(
            log.clone(),
            "r",
            |_| true,
            next
        ),];

        chain.handle("hello".to_string());
        assert_eq!(*log.borrow(), vec!["r:hello"]);
    }

    #[test]
    fn single_handler_chain_matches() {
        let log = Rc::new(RefCell::new(Vec::new()));

        let chain = chain![|next| Recorder::new(log.clone(), "r", |req| req == "match", next),];

        chain.handle("match".to_string());
        assert_eq!(*log.borrow(), vec!["r:match"]);
    }

    #[test]
    fn single_handler_chain_falls_through() {
        let log = Rc::new(RefCell::new(Vec::new()));

        let chain = chain![|next| Recorder::new(log.clone(), "r", |req| req == "match", next),];

        chain.handle("no match".to_string());
        assert!(log.borrow().is_empty());
    }

    #[test]
    fn multi_handler_chain_routes_correctly() {
        let log = Rc::new(RefCell::new(Vec::new()));

        let chain = chain![
            |next| Recorder::new(log.clone(), "A", |req| req == "A", next),
            |next| Recorder::new(log.clone(), "B", |req| req == "B", next),
        ];

        chain.handle("B".to_string());
        chain.handle("A".to_string());
        assert_eq!(*log.borrow(), vec!["B:B", "A:A"]);
    }

    #[test]
    fn unmatched_request_falls_through_entire_chain() {
        let log = Rc::new(RefCell::new(Vec::new()));

        let chain = chain![
            |next| Recorder::new(log.clone(), "A", |req| req == "A", next),
            |next| Recorder::new(log.clone(), "B", |req| req == "B", next),
        ];

        chain.handle("C".to_string());
        assert!(log.borrow().is_empty());
    }

    #[test]
    fn first_matching_handler_wins() {
        let log = Rc::new(RefCell::new(Vec::new()));

        let chain = chain![
            |next| Recorder::new(log.clone(), "first", |_| true, next),
            |next| Recorder::new(log.clone(), "second", |_| true, next),
        ];

        chain.handle("x".to_string());
        assert_eq!(*log.borrow(), vec!["first:x"]);
    }
}
