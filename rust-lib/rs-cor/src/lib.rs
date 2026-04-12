#[macro_export]
macro_rules! chain {
    // Base Case
    ($head:expr $(,)?) => {
        $head
    };

    ($head:expr, $($rest:expr),+ $(,)?) => {{
          let mut head = $head;
          let tail = chain!($($rest),+);
          head.set_next(Box::new(tail));
          head
     }};
}
