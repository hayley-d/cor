extern crate self as cor;

pub use macros_lib::{Handler, handler};

pub trait Handler<T> {
    fn handle(&self, request: T);
}

pub struct NilHandler;

impl<T> Handler<T> for NilHandler {
    fn handle(&self, _: T) {}
}

#[derive(Handler)]
pub struct BaseHandler<T, N: Handler<T>> {
    pub next: N,
    _phantom: std::marker::PhantomData<T>,
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

#[macro_export]
macro_rules! chain {
    ($last:expr $(,)?) => {
        $last(::cor::NilHandler::new())
    };

    ($head:expr, $($rest:expr),+ $(,)?) => {{
        $head(chain!($($rest),+))
    }};
}
