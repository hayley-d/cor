# rs-cor-macros

Proc macros for the [rs-cor](https://github.com/hayley-d/cor) Chain of Responsibility library.

This crate provides the `#[handler]` attribute macro and `#[derive(Handler)]` derive macro used by the `rs-cor` library. It is not intended to be used directly. Use `rs-cor` instead, which re-exports both macros.

## Macros

### `#[handler]`

Applied to a struct with a single type parameter `T`. Generates the handler struct scaffolding:

- An additional generic `N: Handler<T>` for the next handler in the chain
- A `next: N` field and a `_phantom: PhantomData<T>` field
- A `new(<user fields>, next)` constructor — user-defined fields are preserved and appear as positional arguments before `next`

You then provide the routing logic by writing your own `Handler<T>` implementation.

```rust
use cor::{Handler, chain, handler};

#[handler]
struct Logger<T> {}

impl<N: Handler<String>> Handler<String> for Logger<String, N> {
    fn handle(&self, request: String) {
        if !request.is_empty() {
            println!("{}", request);
        } else {
            self.next.handle(request);
        }
    }
}

let chain = chain![
    |next| Logger::new(next),
];
```

### `#[derive(Handler)]`

Derives a pass-through `Handler<T>` implementation that forwards every request to `self.next`. The struct must have a type parameter `T` (the request type) and a field named `next` whose type implements `Handler<T>`.

```rust
use cor::Handler;

#[derive(Handler)]
struct Passthrough<T, N: Handler<T>> {
    next: N,
    _phantom: std::marker::PhantomData<T>,
}
```

## License

GPL-3.0-only. See [LICENSE](../../LICENSE) for details.
