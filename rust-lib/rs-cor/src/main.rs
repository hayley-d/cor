use cor::{Handler, chain, handler};

#[handler]
struct ConcreteHandlerA<T> {}

#[handler]
struct ConcreteHandlerB<T> {}

fn main() {
    let chain = chain![
        |next| ConcreteHandlerA::new(|req: &String| req == "A", |req| println!("Handled A: {}", req), next),
        |next| ConcreteHandlerB::new(|req: &String| req == "B", |req| println!("Handled B: {}", req), next),
    ];

    chain.handle("B".to_string());
}
