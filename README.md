# cor

A Rust library for building Chain of Responsibility pipelines using statically-typed, generic handler chains.

Handlers are composed at compile time with zero dynamic dispatch overhead in the chain structure itself. Each handler either processes a request or forwards it to the next handler in the chain.

## Usage

Add `cor` as a dependency (path-based for now):

```toml
[dependencies]
cor = { path = "../cor" }
```

### Defining handlers

Use the `#[handler]` attribute macro on a struct with a single type parameter. The macro generates the struct fields, constructor, and `Handler<T>` implementation for you:

```rust
use cor::{Handler, chain, handler};

#[handler]
struct AuthHandler<T> {}

#[handler]
struct LogHandler<T> {}
```

Each generated handler gets a `new(condition, on_match, next)` constructor:
- `condition` — a closure `Fn(&T) -> bool` that decides whether this handler processes the request
- `on_match` — a closure `Fn(&T)` that runs when the condition is true
- `next` — the next handler in the chain (passed automatically by the `chain!` macro)

### Building a chain

Use the `chain!` macro to compose handlers. Each entry is a closure that receives the next handler and returns a configured handler:

```rust
fn main() {
    let chain = chain![
        |next| AuthHandler::new(
            |req: &String| req.starts_with("auth:"),
            |req| println!("Authenticated: {}", req),
            next,
        ),
        |next| LogHandler::new(
            |req: &String| req.starts_with("log:"),
            |req| println!("Logged: {}", req),
            next,
        ),
    ];

    chain.handle("log:hello".to_string());  // prints: Logged: log:hello
    chain.handle("auth:admin".to_string()); // prints: Authenticated: auth:admin
    chain.handle("other".to_string());      // no output, falls through to NilHandler
}
```

The `chain!` macro nests handlers right-to-left, terminating with a `NilHandler` that silently drops unhandled requests.

### Custom handler structs

You can add extra fields to handler structs. The macro will preserve them and add the chain machinery alongside:

```rust
#[handler]
struct ThresholdHandler<T> {
    pub threshold: u32,
}
```

### Derive macro

For pass-through handlers that only forward to the next handler (like `BaseHandler`), use `#[derive(Handler)]` instead:

```rust
#[derive(Handler)]
struct MyPassthrough<T, N: Handler<T>> {
    next: N,
    _phantom: std::marker::PhantomData<T>,
}
```

## License

Copyright (c) 2026 Hayley Dodkins. All rights reserved. See [LICENSE](LICENSE) for details.

## Contact

Hayley Dodkins — u21528790@tuks.co.za
