use cor::{BaseHandler, Handler, chain, handler};

#[handler]
struct ConcreteHandlerA<T> {
    next: Option<Box<dyn Handler<T>>>,
}

#[handler]
struct ConcreteHandlerB<T> {
    next: Option<Box<dyn Handler<T>>>,
}

fn main() {
    let handler = BaseHandler::<String>::new();
    let handler_1 =
        ConcreteHandlerA::<String>::new(|req| req == "A", |req| println!("Handled A: {}", req));
    let handler_2 =
        ConcreteHandlerB::<String>::new(|req| req == "B", |req| println!("Handled B: {}", req));

    let chain = chain![handler, handler_1, handler_2];

    chain.handle("B".to_string());
}
