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
//! use cor::{Handler, NilHandler, chain, append_chain, handler};
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
//!     Log
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
//! impl<N: Handler<LogRequest>> Handler<LogRequest> for ErrorHandler<LogRequest, N>
//! {
//!     fn handle(&self, request: LogRequest) {
//!         if request.level == LogLevel::Error {
//!             println!("[ERROR] {}", request.message);
//!         } else {
//!             self.next.handle(request);
//!         }
//!     }
//! }
//!
//! #[handler]
//! struct LogHandler<T> {}
//!
//! impl<N: Handler<LogRequest>> Handler<LogRequest> for LogHandler<LogRequest, N> {
//!     fn handle(&self, request: LogRequest) {
//!         if request.level == LogLevel::Log {
//!             println!("[LOG] {}", request.message);
//!         } else {
//!             self.next.handle(request);
//!         }
//!     }
//! }
//!
//! let base_handler = NilHandler::new();
//! let logger = chain![InfoHandler, WarningHandler, ErrorHandler];
//!
//! logger.handle(LogRequest {
//!     level: LogLevel::Info,
//!     message: "Server started on port 8080".into(),
//! });
//!
//! let extended_chain = append_chain![LogHandler; logger];
//!
//! extended_chain.handle(LogRequest {
//!     level: LogLevel::Log,
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
//! The [`chain!`] macro composes handler types right-to-left, terminating with a
//! `base_handler` variable (typically a [`NilHandler`]) that the caller brings
//! into scope. Each entry is a handler type whose `new` takes the next handler
//! as its final argument, producing a fully nested type at compile time.

extern crate self as cor;

pub use macros_lib::{Handler, append_chain, chain, handler};

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

pub trait Linker<T, N: Handler<T>> {
    type Output: Handler<T>;
    fn append(self, new_handler: N) -> Self::Output;
}

/// A terminal handler that discards any request it receives.
///
/// `NilHandler` is the conventional tail of a chain: bind it to a local
/// `base_handler` and the [`chain!`] macro will nest handlers on top of it.
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

impl<T, N: Handler<T>> Linker<T, N> for NilHandler {
    type Output = N;

    fn append(self, new_handler: N) -> Self::Output {
        new_handler
    }
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
    fn chain_macro_composes_handler_types() {
        let base_handler: NilHandler = NilHandler::new();
        let chain = chain![BaseHandler, BaseHandler];
        chain.handle("hello".to_string());
    }

    #[test]
    fn base_handler_forwards_to_next() {
        let log = Rc::new(RefCell::new(Vec::new()));

        let chain = BaseHandler::new(Recorder::new(log.clone(), "r", |_| true, NilHandler::new()));

        chain.handle("hello".to_string());
        assert_eq!(*log.borrow(), vec!["r:hello"]);
    }

    #[test]
    fn single_handler_chain_matches() {
        let log = Rc::new(RefCell::new(Vec::new()));

        let chain = Recorder::new(log.clone(), "r", |req| req == "match", NilHandler::new());

        chain.handle("match".to_string());
        assert_eq!(*log.borrow(), vec!["r:match"]);
    }

    #[test]
    fn single_handler_chain_falls_through() {
        let log = Rc::new(RefCell::new(Vec::new()));

        let chain = Recorder::new(log.clone(), "r", |req| req == "match", NilHandler::new());

        chain.handle("no match".to_string());
        assert!(log.borrow().is_empty());
    }

    #[test]
    fn multi_handler_chain_routes_correctly() {
        let log = Rc::new(RefCell::new(Vec::new()));

        let chain = Recorder::new(
            log.clone(),
            "A",
            |req| req == "A",
            Recorder::new(log.clone(), "B", |req| req == "B", NilHandler::new()),
        );

        chain.handle("B".to_string());
        chain.handle("A".to_string());
        assert_eq!(*log.borrow(), vec!["B:B", "A:A"]);
    }

    #[test]
    fn unmatched_request_falls_through_entire_chain() {
        let log = Rc::new(RefCell::new(Vec::new()));

        let chain = Recorder::new(
            log.clone(),
            "A",
            |req| req == "A",
            Recorder::new(log.clone(), "B", |req| req == "B", NilHandler::new()),
        );

        chain.handle("C".to_string());
        assert!(log.borrow().is_empty());
    }

    #[test]
    fn first_matching_handler_wins() {
        let log = Rc::new(RefCell::new(Vec::new()));

        let chain = Recorder::new(
            log.clone(),
            "first",
            |_| true,
            Recorder::new(log.clone(), "second", |_| true, NilHandler::new()),
        );

        chain.handle("x".to_string());
        assert_eq!(*log.borrow(), vec!["first:x"]);
    }
}
