# rs-cor

A compile-time Chain of Responsibility pattern library for Rust.

Handlers are composed into statically-typed, generic chains with zero dynamic dispatch overhead in the chain structure. Each handler either processes a request or forwards it to the next handler.

## Installation

```toml
[dependencies]
rs-cor = "0.2.0"
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

impl<N: Handler<LogRequest>> Handler<LogRequest> for InfoHandler<LogRequest, N> {
    fn handle(&self, request: LogRequest) {
        if request.level == LogLevel::Info {
            println!("[INFO] {}", request.message);
        } else {
            self.next.handle(request);
        }
    }
}

#[handler]
struct WarningHandler<T> {}

impl<N: Handler<LogRequest>> Handler<LogRequest> for WarningHandler<LogRequest, N> {
    fn handle(&self, request: LogRequest) {
        if request.level == LogLevel::Warning {
            println!("[WARN] {}", request.message);
        } else {
            self.next.handle(request);
        }
    }
}

#[handler]
struct ErrorHandler<T> {}

impl<N: Handler<LogRequest>> Handler<LogRequest> for ErrorHandler<LogRequest, N> {
    fn handle(&self, request: LogRequest) {
        if request.level == LogLevel::Error {
            println!("[ERROR] {}", request.message);
        } else {
            self.next.handle(request);
        }
    }
}

fn main() {
    let logger = chain![
        |next| InfoHandler::new(next),
        |next| WarningHandler::new(next),
        |next| ErrorHandler::new(next),
    ];

    logger.handle(LogRequest {
        level: LogLevel::Info,
        message: "Server started on port 8080".into(),
    });
}
```

## How It Works

The `#[handler]` attribute macro generates the struct scaffolding:

- A `next: N` field and a `_phantom: PhantomData<T>` field
- A `new(<user fields>, next)` constructor

You then write your own `Handler<T>` implementation to define the routing logic — typically inspecting the request and either handling it or calling `self.next.handle(request)` to forward.

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
| `#[handler]` | Attribute macro that generates handler struct scaffolding (`next` field, constructor) |
| `#[derive(Handler)]` | Derive macro for pass-through handlers that forward to `next` |
| `chain!` | Composes handlers into a linked chain |

## Custom Handler Fields

You can add extra fields to handler structs. The macro preserves them and includes them as positional arguments to `new` in declaration order, followed by `next`:

```rust
use cor::{Handler, handler};

#[handler]
struct ThresholdHandler<T> {
    pub threshold: u32,
}

impl<N: Handler<u32>> Handler<u32> for ThresholdHandler<u32, N> {
    fn handle(&self, request: u32) {
        if request >= self.threshold {
            println!("over threshold: {}", request);
        } else {
            self.next.handle(request);
        }
    }
}

// Constructed as: ThresholdHandler::new(100, next)
```

## Manual Handler Implementation

For full control, implement the `Handler<T>` trait directly without the attribute macro:

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
