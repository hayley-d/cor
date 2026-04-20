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

Use the `#[handler]` attribute macro on a struct with a single type parameter. The macro generates the struct fields and constructor; you provide the `Handler<T>` implementation with your routing logic:

```rust
use cor::{Handler, chain, handler};

#[handler]
struct AuthHandler<T> {}

impl<N: Handler<String>> Handler<String> for AuthHandler<String, N> {
    fn handle(&self, request: String) {
        if request.starts_with("auth:") {
            println!("Authenticated: {}", request);
        } else {
            self.next.handle(request);
        }
    }
}

#[handler]
struct LogHandler<T> {}

impl<N: Handler<String>> Handler<String> for LogHandler<String, N> {
    fn handle(&self, request: String) {
        if request.starts_with("log:") {
            println!("Logged: {}", request);
        } else {
            self.next.handle(request);
        }
    }
}
```

Each generated handler gets a `new(<user fields>, next)` constructor, where `next` is the next handler in the chain (passed automatically by the `chain!` macro).

### Building a chain

Use the `chain!` macro to compose handlers. Each entry is a closure that receives the next handler and returns a configured handler:

```rust
fn main() {
    let chain = chain![
        |next| AuthHandler::new(next),
        |next| LogHandler::new(next),
    ];

    chain.handle("log:hello".to_string());  // prints: Logged: log:hello
    chain.handle("auth:admin".to_string()); // prints: Authenticated: auth:admin
    chain.handle("other".to_string());      // no output, falls through to NilHandler
}
```

The `chain!` macro nests handlers right-to-left, terminating with a `NilHandler` that silently drops unhandled requests.

### Custom handler fields

You can add extra fields to handler structs. They are preserved and appear as positional arguments on `new` in declaration order, followed by `next`:

```rust
#[handler]
struct ThresholdHandler<T> {
    pub threshold: u32,
}

// Constructed as: ThresholdHandler::new(100, next)
```

### Derive macro

For pass-through handlers that only forward to the next handler, use `#[derive(Handler)]` instead:

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
