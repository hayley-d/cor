# rs-cor

A compile-time Chain of Responsibility pattern library for Rust.

Handlers are composed into statically-typed, generic chains with zero dynamic dispatch overhead in the chain structure. Each handler either processes a request or forwards it to the next handler.

## Installation

```toml
[dependencies]
rs-cor = "0.1.1"
```

## Quick Start

```rust
use cor::{Handler, chain, handler};

#[derive(Clone)]
struct LogRequest {
    level: LogLevel,
    message: String,
}

#[derive(Clone, PartialEq)]
enum LogLevel {
    Info,
    Warning,
    Error,
}

#[handler]
struct InfoHandler<T> {}

#[handler]
struct WarningHandler<T> {}

#[handler]
struct ErrorHandler<T> {}

fn main() {
    let logger = chain![
        |next| InfoHandler::new(
            |req: &LogRequest| req.level == LogLevel::Info,
            |req| println!("[INFO] {}", req.message),
            next,
        ),
        |next| WarningHandler::new(
            |req: &LogRequest| req.level == LogLevel::Warning,
            |req| println!("[WARN] {}", req.message),
            next,
        ),
        |next| ErrorHandler::new(
            |req: &LogRequest| req.level == LogLevel::Error,
            |req| println!("[ERROR] {}", req.message),
            next,
        ),
    ];

    logger.handle(LogRequest {
        level: LogLevel::Info,
        message: "Server started on port 8080".into(),
    });
}
```

## How It Works

The `#[handler]` attribute macro generates a handler struct with:

- A `condition: Box<dyn Fn(&T) -> bool>` field that decides whether the handler processes the request
- An `on_match: Box<dyn Fn(&T)>` field that runs when the condition is true
- A `next: N` field for the next handler in the chain
- A `new(condition, on_match, next)` constructor
- A `Handler<T>` implementation that calls `on_match` when `condition` returns true, otherwise forwards to `next`

The `chain!` macro composes handler constructors right-to-left, terminating with a `NilHandler` that silently drops unhandled requests. Each entry is a closure receiving the next handler and returning a configured handler.

## Key Types

| Type | Description |
|------|-------------|
| `Handler<T>` | Core trait with `handle(&self, request: T)` |
| `NilHandler` | Terminal handler that discards requests (auto-placed at chain tail) |
| `BaseHandler<T, N>` | Pass-through handler that forwards every request to `next` |

## Macros

| Macro | Description |
|-------|-------------|
| `#[handler]` | Attribute macro that generates a condition-based handler struct |
| `#[derive(Handler)]` | Derive macro for pass-through handlers that forward to `next` |
| `chain!` | Composes handlers into a linked chain |

## Custom Handler Fields

You can add extra fields to handler structs. The macro preserves them alongside the generated chain machinery:

```rust
#[handler]
struct ThresholdHandler<T> {
    pub threshold: u32,
}
```

## Manual Handler Implementation

For full control, implement the `Handler<T>` trait directly:

```rust
use cor::Handler;

struct Upper<N: Handler<String>> {
    next: N,
}

impl<N: Handler<String>> Handler<String> for Upper<N> {
    fn handle(&self, request: String) {
        if request.chars().all(|c| c.is_uppercase()) {
            println!("ALL CAPS: {}", request);
        } else {
            self.next.handle(request);
        }
    }
}
```

## License

GPL-3.0-only. See [LICENSE](../../LICENSE) for details.
