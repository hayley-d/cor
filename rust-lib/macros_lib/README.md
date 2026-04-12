# rs-cor-macros

Proc macros for the [rs-cor](https://github.com/hayley-d/cor) Chain of Responsibility library.

This crate provides the `#[handler]` attribute macro and `#[derive(Handler)]` derive macro used by the `rs-cor` library. It is not intended to be used directly. Use `rs-cor` instead, which re-exports both macros.

## Macros

### `#[handler]`

Applied to a struct with a single type parameter `T`. Generates:

- An additional generic `N: Handler<T>` for the next handler in the chain
- Fields: `next: N`, `condition: Box<dyn Fn(&T) -> bool>`, `on_match: Box<dyn Fn(&T)>`
- A `new(condition, on_match, next)` constructor
- A `Handler<T>` implementation that calls `on_match` when `condition` returns true, otherwise forwards to `next`

Any user-defined fields are preserved.

```rust
use cor::{handler, Handler, chain};

#[handler]
struct Logger<T> {}

let chain = chain![
    |next| Logger::new(|req: &String| !req.is_empty(), |req| println!("{}", req), next),
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
