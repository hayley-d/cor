extern crate self as cor;

pub use macros_lib::{Handler, handler};

pub trait Handler<T> {
    fn set_next(&mut self, next: Box<dyn Handler<T>>);
    fn handle(&self, request: T);
}

#[derive(Handler)]
pub struct BaseHandler<T> {
    pub next: Option<Box<dyn Handler<T>>>,
}

impl<T> BaseHandler<T> {
    pub fn new() -> Self {
        BaseHandler { next: None }
    }
}

impl<T> Default for BaseHandler<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! chain {
    ($head:expr $(,)?) => {
        $head
    };
    ($head:expr, $($rest:expr),+ $(,)?) => {{
        let mut head = $head;
        let tail = $crate::chain!($($rest),+);
        $crate::Handler::set_next(&mut head, Box::new(tail));
        head
    }};
}
