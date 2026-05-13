# Rust + Leptos: Server Architecture Study Notes

A complete reference built from a layered counter mini-project, designed to internalize the patterns used in a real-world Rust/Leptos codebase (`services → server_logic → server_fns → component`).

---

## Table of Contents

1. [The Mini Project](#1-the-mini-project)
2. [Borrowing and Lifetimes](#2-borrowing-and-lifetimes)
3. [Server-Side in Rust: Mental Model](#3-server-side-in-rust-mental-model)
4. [AppState and Arc](#4-appstate-and-arc)
5. [Leptos `#[component]`](#5-leptos-component)
6. [Leptos `#[server]`: The Bridge](#6-leptos-server-the-bridge)
7. [Layered Architecture Mapping](#7-layered-architecture-mapping)
8. [`config.rs` — Dissected](#8-configrs--dissected)
9. [`services.rs` — Dissected](#9-servicesrs--dissected)
10. [Arc Granularity Without Mutex](#10-arc-granularity-without-mutex)
11. [`server_logic.rs` — Dissected](#11-server_logicrs--dissected)
12. [`main.rs` — Dissected](#12-mainrs--dissected)
13. [`server_fns.rs` — Dissected](#13-server_fnsrs--dissected)
14. [`app.rs` — Component Layer Dissected](#14-apprs--component-layer-dissected)
15. [The Full Lifecycle](#15-the-full-lifecycle)

---

## 1. The Mini Project

A persistent counter on the server, mirroring the architecture:

```
mini_counter/
└── src/
    ├── main.rs            // entry point, wires AppState
    ├── app.rs             // <Counter/> component
    ├── state.rs           // AppState definition
    ├── services.rs        // CounterService (lowest layer)
    ├── server_logic.rs    // business rules
    ├── server_fns.rs      // #[server] entry points
    └── config.rs          // env-loaded config
```

This structure mirrors the production project's `services → server_logic → server_fns → component` flow.

---

## 2. Borrowing and Lifetimes

### The mental model

Every value has exactly **one owner**. The owner is responsible for cleanup. Borrowing lets others *look* without taking ownership. Lifetimes are just the compiler's proof that no borrow outlives what it points to.

### Three rules

- One owner at a time
- Many `&T` *or* one `&mut T`, never both
- A borrow can't outlive its referent

### Why this matters for servers

Async tasks run concurrently and can outlive any single function call. A `&T` would need a lifetime that survives across `.await` points and across tasks — almost always impossible to express.

So on the server side you rarely pass `&T`. You pass:
- *Owned* values (`String`, `Vec<T>`)
- *Shared owned* values (`Arc<T>`)

Lifetimes largely disappear because everything is `'static` (owned or shared).

### Rule of thumb

If data lives across an `.await`, use **owned types** or **`Arc`**. References are for synchronous, scoped work — inside a single function, no awaits in between.

---

## 3. Server-Side in Rust: Mental Model

A Rust web server is a program that:

1. Holds long-lived state
2. Listens on a port
3. For each request, the async runtime (tokio) spawns a task
4. The task reads, computes, and writes a response

Many request-tasks plus the runtime are running concurrently, all needing the same shared state. **That's where `Arc` enters.**

### In Leptos + Axum

```
HTTP request 
  → Axum router 
    → matches a route 
      → calls a handler (static file | SSR render | server fn)
        → handler reads AppState (an Arc<...>) 
          → calls your code 
            → returns response
```

The runtime owns the server, the server owns the router, the router holds AppState (cloneable Arc), each request handler gets a clone.

### Critical insight: dual compilation

**Your binary is compiled twice:**

- Once natively for the **server** (with `feature = "ssr"`)
- Once to **WASM** for the **browser** (with `feature = "hydrate"`)

Same source files. `cfg`s pick which code is included where. This is what makes `#[server]` functions possible.

---

## 4. AppState and Arc

```rust
#[derive(Clone)]
pub struct AppState {
    pub counter: Arc<CounterService>,
}
```

### Mental model

AppState is the *handle* to your shared resources. It's small, cheap to clone, and every request handler gets its own clone. The clones all point to the same underlying data because the inside is `Arc<...>`.

### Why `Arc` instead of `&'static`?

`&'static` requires the value be known at compile time or intentionally leaked. Real state (DB pools, RPC providers, configs from env) is created at runtime. `Arc` is a runtime-counted reference; when the last clone drops, the data drops. It's atomic, so it crosses thread boundaries safely.

### Why `Mutex` (or `RwLock`) inside?

`Arc<T>` gives shared *immutable* access. To mutate, you need **interior mutability** with synchronization. The pattern `Arc<Mutex<T>>` means "shared mutable state across tasks."

### Advantages

- Cheap clone (atomic increment)
- Type system enforces lifetime correctness
- Works seamlessly across `.await` points

---

## 5. Leptos `#[component]`

```rust
#[component]
pub fn Counter(initial: i32) -> impl IntoView {
    let (count, set_count) = signal(initial);

    view! {
        <button on:click=move |_| set_count.update(|n| *n += 1)>
            "Count: " {count}
        </button>
    }
}
```

### What the macro does

Generates a `CounterProps` struct with field `initial`, a builder for it, and rewrites your function to take `CounterProps` and destructure it. So `<Counter initial=5/>` desugars to `Counter(CounterProps::builder().initial(5).build())`.

### Signals: the reactive primitive

Mental model: a signal is a value + a list of subscribers. When you *read* it inside a reactive scope (like a `view!` closure), that scope subscribes. When you *write* it, all subscribers re-run.

`signal(0)` returns `(ReadSignal, WriteSignal)`. The split is enforced at the type level.

### The biggest gotcha (vs React)

Reactivity is **fine-grained**, not component-level. Your component body runs **once**. Only the closures inside the `view!` macro that read a given signal re-run when it changes — usually just a single text node or attribute.

This is why you see `move ||` everywhere: those closures are the reactive units.

---

## 6. Leptos `#[server]`: The Bridge

```rust
#[server(IncrementCounter, "/api")]
pub async fn increment_counter(by: i32) -> Result<i32, ServerFnError> {
    use leptos::prelude::*;
    let state = expect_context::<AppState>();
    let mut guard = state.counter.lock().unwrap();
    *guard += by;
    Ok(*guard)
}
```

### What the macro emits

Same source, compiled twice. The macro emits *both sides* from the same definition.

**Server build (`feature = "ssr"`):** the function body you wrote is kept as-is. Additionally, an HTTP handler is generated and registered at `/api/increment_counter`.

**Client build (`feature = "hydrate"`, WASM):** the function body is *replaced* by a stub. The stub serializes args, issues `fetch("/api/increment_counter", ...)`, awaits the response, deserializes into `Result<i32, ServerFnError>`.

### The flow

```
Component:                      Network:                Server handler:
increment_counter(1).await
  │
  ▼ (macro stub)
serialize args ─── POST /api/increment_counter ──▶ deserialize args
                                                    │
                                                    ▼
                                                 your real fn body
                                                    │
deserialize response ◀── HTTP 200 + body ─── serialize result
  │
  ▼
return Result<i32, ServerFnError>
```

### Why this design is so nice

- Write one function, get RPC for free
- Types are checked end-to-end — change the signature and both sides fail to compile
- During SSR, the server skips the HTTP loop entirely (direct function call)

### Calling from a component

```rust
#[component]
pub fn Counter() -> impl IntoView {
    let inc = Action::new(|by: &i32| {
        let by = *by;
        async move { increment_counter(by).await }
    });
    let count = move || inc.value().get().and_then(|r| r.ok()).unwrap_or(0);

    view! {
        <button on:click=move |_| { inc.dispatch(1); }>
            "Count: " {count}
        </button>
    }
}
```

`Action` wraps the async server call into a signal-aware container. `dispatch` triggers it; `value()` gives a signal that updates when the future resolves.

---

## 7. Layered Architecture Mapping

| Layer | Purpose | Knows about | Doesn't know about |
|---|---|---|---|
| **Services** | Operations on external systems | External APIs, DBs | Business rules |
| **Server logic** | Business rules, orchestration | Services | HTTP, Leptos |
| **Server functions** | HTTP boundary | AppState, server logic | UI |
| **Components** | UI, reactivity | Server functions | Network details |

### The payoff

- Server logic can be unit-tested with mocked services and zero HTTP
- Server functions just glue HTTP to logic
- Components just glue UI to server functions
- Each layer has one job and only one reason to change

---

## 8. `config.rs` — Dissected

```rust
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub initial_count: i32,
    pub max_increment: i32,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("invalid value for {0}: {1}")]
    Invalid(String, String),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let initial_count = env::var("INITIAL_COUNT")
            .unwrap_or_else(|_| "0".to_string())
            .parse::<i32>()
            .map_err(|e| ConfigError::Invalid("INITIAL_COUNT".into(), e.to_string()))?;

        let max_increment = env::var("MAX_INCREMENT")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<i32>()
            .map_err(|e| ConfigError::Invalid("MAX_INCREMENT".into(), e.to_string()))?;

        Ok(Self { initial_count, max_increment })
    }
}
```

### Why owned types and not references

The fields are plain `i32`. There's no `&'a i32` or `&str`. This is deliberate: `Config` is a *self-contained* value that owns its data.

If we had `&'a str` fields, the struct would need a lifetime parameter `Config<'a>`, and every type holding it would need to thread that lifetime through. That's viral.

**General rule:** for data structures stored in long-lived things (like AppState), prefer **owned types**. `String` over `&str`, `i32` over `&i32`, `Vec<T>` over `&[T]`. Borrows are for arguments and locals, not fields.

### Why `#[derive(Debug, Clone)]`

- `Debug` is for `{:?}` formatting. Zero runtime cost if not used. Always derive on your own types.
- `Clone` is interesting. `Config` will be cloned into AppState; cloning two `i32`s is essentially free.

**First key principle:** *`Arc` is not the default. Owned + `Clone` is the default. You only reach for `Arc` when cloning is expensive or you need shared mutable state.*

### `from_env` line by line

- `env::var("INITIAL_COUNT")` returns `Result<String, VarError>`
- `.unwrap_or_else(|_| "0".to_string())` is **lazy** — only allocates default if env var missing
- `.parse::<i32>()` returns `Result<i32, ParseIntError>` (turbofish makes target type explicit)
- `.map_err(|e| ConfigError::Invalid(...))` translates "their" error to "our" error
- `?` propagates: on `Err`, return early; on `Ok`, unwrap

### Layer-specific errors

Every layer of your architecture should have its own error type and translate at the boundary. That way `ConfigError` doesn't leak `ParseIntError` and downstream errors don't leak upstream details. **Errors are part of your API.**

### Why no `Arc<Config>` here

For tiny `Copy`-friendly configs, plain `Clone` wins. For configs holding `reqwest::Client` (already Arc'd internally) or large `String`s, `Arc<Config>` starts paying off.

---

## 9. `services.rs` — Dissected

```rust
use std::sync::{Arc, Mutex};
use crate::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum CounterError {
    #[error("increment {0} exceeds maximum {1}")]
    ExceedsMax(i32, i32),
    #[error("counter mutex poisoned")]
    Poisoned,
}

#[derive(Clone)]
pub struct CounterService {
    state: Arc<Mutex<i32>>,
    max_increment: i32,
}

impl CounterService {
    pub fn new(config: &Config) -> Self {
        Self {
            state: Arc::new(Mutex::new(config.initial_count)),
            max_increment: config.max_increment,
        }
    }

    pub fn get(&self) -> Result<i32, CounterError> {
        let guard = self.state.lock().map_err(|_| CounterError::Poisoned)?;
        Ok(*guard)
    }

    pub fn add(&self, by: i32) -> Result<i32, CounterError> {
        if by > self.max_increment {
            return Err(CounterError::ExceedsMax(by, self.max_increment));
        }
        let mut guard = self.state.lock().map_err(|_| CounterError::Poisoned)?;
        *guard += by;
        Ok(*guard)
    }
}
```

### `Arc<Mutex<i32>>`: layered shared-mutable

Read it from outside in:
- `Arc<...>` → "shared ownership across threads"
- `Mutex<...>` → "one writer at a time, with synchronization"
- `i32` → the actual data

**Why two wrappers?**

- `Arc<i32>` alone gives shared *immutable* access. Many tasks read, none mutate.
- `Mutex<i32>` alone can't be cloned to share across tasks (clones would be independent locks).
- `Arc<Mutex<i32>>` composes them: shareable AND mutable through the share.

The Mutex enforces "one writer at a time" at runtime, because static analysis can't see across tasks.

**This is the single most important pattern for shared state in Rust async code.**

### Why the Arc is *inside* the service, not around it

`CounterService` is `Clone`, NOT wrapped in `Arc<Mutex<CounterService>>`.

If we'd wrapped the whole service: every operation — even reading immutable `max_increment` — would acquire one giant lock. Concurrent calls would serialize entirely.

By putting `Arc<Mutex<...>>` only around the *truly shared mutable* piece:
- Immutable fields read without any lock
- Lock held only briefly during actual mutation
- **Lock granularity matters**

### `&self` with mutation: interior mutability

```rust
pub fn add(&self, by: i32) -> Result<i32, CounterError> {
    ...
    *guard += by;  // mutating!
}
```

Method takes `&self` (shared ref) yet mutates inner integer. How?

Because `Mutex` doesn't give direct mutation — it gives a `MutexGuard` whose existence comes from acquiring the lock at runtime. The lock provides a different proof of exclusivity than the borrow checker.

This pattern is **interior mutability**. Types enabling it (`Mutex`, `RwLock`, `RefCell`, `Cell`, `Atomic*`) have a special unsafe core (`UnsafeCell`) that opts out of the borrow checker's exclusivity rule and replaces it with their own runtime check.

**Why use `&self` mutation?** Because it lets *many places* hold a reference to the service simultaneously — including across `Arc::clone` — and any of them can mutate. With `&mut self`, only one place could ever call `add`.

### The `MutexGuard` and `*guard` syntax

`self.state.lock()` returns `Result<MutexGuard<'_, i32>, PoisonError<...>>`.

`MutexGuard<i32>` is a smart pointer implementing `Deref` and `DerefMut` for `i32`. So `*guard` reads the inner i32, and `*guard += by` mutates it.

When `guard` goes out of scope, its `Drop` impl runs and releases the lock. **This is RAII** — the lock is tied to the guard's lifetime; you can't forget to unlock.

### Why returning copy, not reference

`Ok(*guard)` returns a *copy* (because `i32: Copy`), not a reference. Returning `&i32` would require the reference to outlive the guard, but the guard drops at function end. The compiler stops you. For non-Copy types, you'd `.clone()` before returning.

### Mutex poisoning

A `Mutex` becomes "poisoned" if a thread panics while holding it. The thinking: data may be inconsistent.

In practice, `.lock().unwrap()` is fine for many apps. Mapping it to a domain error (as we do) lets the upper layer decide.

### `std::sync::Mutex` vs `tokio::sync::Mutex`

**The rule: never hold a `std::sync::Mutex` guard across an `.await`.**

Why: `std::sync::Mutex` blocks the OS thread. If a tokio task holds it and awaits, the task gets parked but still holds the lock on the OS thread the executor uses for *other* tasks. Deadlock or massive latency.

`tokio::sync::Mutex` is async-aware — its `lock()` returns a future. Holding it across `.await` is fine.

**So:**
- Cheap synchronous critical sections (incrementing an integer, pushing to a Vec) → `std::sync::Mutex` (faster)
- Critical sections that need to await → `tokio::sync::Mutex`

### Method signatures: don't return references

```rust
pub fn get(&self) -> Result<i32, CounterError>  // returns i32, not &i32
```

Returning a reference would require it to outlive the lock guard — impossible. **Method return values should be owned**, especially for anything coming out from behind a lock.

### Private fields

Both `state` and `max_increment` are private. From outside, you can only interact via `new`, `get`, `add`. Benefits:
- Can never accidentally lock the mutex twice from outside
- Can swap `Arc<Mutex<i32>>` for `Arc<RwLock<i32>>` or DB without touching callers
- Enforce invariants

**Encapsulation in Rust isn't ceremony; it's how you give your future self the freedom to change implementations.**

### Tests need no mocks

```rust
#[test]
fn add_within_limit_works() {
    let cfg = Config { initial_count: 0, max_increment: 10 };
    let svc = CounterService::new(&cfg);
    assert_eq!(svc.add(5).unwrap(), 5);
    assert_eq!(svc.add(3).unwrap(), 8);
}
```

No I/O, no mocks needed. When real services have I/O (RPC, subgraph), introduce a trait to mock — but only when needed, not preemptively.

---

## 10. Arc Granularity Without Mutex

When there's no Mutex involved, the granularity argument changes completely.

**With `Mutex`,** granularity was about lock contention — a coarse lock serializes operations that could run in parallel.

**With pure `Arc<T>`** (no interior mutability), there's no contention at all. Every clone bumps a refcount and gives shared read access.

```rust
// Perfectly fine for read-only services
#[derive(Clone)]
pub struct AppState {
    pub counter: Arc<CounterService>,   // ← Arc the whole thing
    pub config: Arc<Config>,
}
```

### When granular Arcs still matter

| Situation | What to do |
|---|---|
| Immutable data, shared everywhere | `Arc<WholeStruct>`, granularity doesn't matter |
| Mutable fields, one big lock | `Arc<Mutex<WholeStruct>>`, works but serializes everything |
| Mutable fields, independent | `Arc<Mutex<Field>>` per field, parallel operations |
| Sub-Arc passed separately to other services | Inner `Arc<T>` per field, clone just that one |

### Independent sharing

The other reason for inner Arcs besides contention: **independent sharing**. If `DbPool` inside `CounterService` also needs to live inside `LoggingService`, you don't want to clone the whole `CounterService` into it. You clone just the `Arc<DbPool>`. That's only possible if the pool is its own Arc.

In production: `Arc<Provider>` shared between `coop_registry` and `coop_deployment` services. That's why it's an inner Arc — for sharing, not mutation.

---

## 11. `server_logic.rs` — Dissected

```rust
use crate::services::{CounterError, CounterService};

#[derive(Debug, thiserror::Error)]
pub enum LogicError {
    #[error("increment must be between 1 and {max}, got {got}")]
    OutOfRange { got: i32, max: i32 },

    #[error("counter would overflow i32")]
    WouldOverflow,

    #[error(transparent)]
    Service(#[from] CounterError),
}

pub fn get_count(counter: &CounterService) -> Result<i32, LogicError> {
    Ok(counter.get()?)
}

pub fn increment(counter: &CounterService, by: i32) -> Result<i32, LogicError> {
    if by < 1 || by > 100 {
        return Err(LogicError::OutOfRange { got: by, max: 100 });
    }

    let current = counter.get()?;
    current
        .checked_add(by)
        .ok_or(LogicError::WouldOverflow)?;

    counter.add(by).map_err(LogicError::Service)
}

pub fn reset(counter: &CounterService, target: i32) -> Result<i32, LogicError> {
    let current = counter.get()?;
    let delta = target - current;

    if delta == 0 {
        return Ok(current);
    }

    counter.add(delta).map_err(LogicError::Service)
}
```

### `#[error(transparent)]` vs `#[error("...")]`

- `#[from]` generates `From<CounterError> for LogicError` impl, making `?` work
- `#[error(transparent)]` delegates `Display` to the inner error

```rust
// transparent: "increment 150 exceeds maximum 100"  (CounterError's own message)
#[error(transparent)]
Service(#[from] CounterError),

// wrapping: "service error: increment 150 exceeds maximum 100"
#[error("service error: {0}")]
Service(#[from] CounterError),
```

### Why `&CounterService` and not `Arc<CounterService>` or owned

| Form | Meaning |
|---|---|
| `counter: CounterService` | Takes ownership, drops at fn end. Wrong for shared resource. |
| `counter: Arc<CounterService>` | Bumps refcount. Useful if spawning tasks. Overkill for sync calls. |
| `counter: &CounterService` | Borrows for this call. Caller keeps ownership. **Correct here.** |

**The rule: take the least powerful thing that gets the job done.** References are cheapest and most constraining. `Arc` is for genuine cross-boundary sharing.

### `checked_add`: arithmetic safety

`i32::checked_add` returns `Option<i32>` — `None` on overflow. Debug builds panic on overflow; release builds wrap silently. Neither is what you want.

**Validate-before-mutate** is a guard pattern: never put state into a bad state and then error.

### Two-step in `increment`: layer separation

`CounterService::add` does its own max-increment check, but it can't do an overflow check because it doesn't know the domain rule about `i32` bounds — that's business logic.

- **Service layer:** "is this a valid operation mechanically?" (within configured limits)
- **Logic layer:** "is this a valid operation in the domain?" (will state make sense after?)

In production: `blockchain` service checks "is this a valid transaction"; server logic checks "is this user allowed to do this with this cooperative."

### `reset`: composed operations belong in logic

`CounterService` has no concept of "reset" or "target value." Only `get` and `add`. That's deliberate — service is a primitive. `reset` orchestrates multiple service calls.

In production: `create_coop` is not a single contract call — it's: validate → deploy contract → register in registry → index in subgraph → return result. None of that orchestration belongs in individual services.

### `map_err` vs `?` with `#[from]`

```rust
counter.get()?                              // ? uses #[from] auto-conversion
counter.add(delta).map_err(LogicError::Service)  // explicit
```

Equivalent when `#[from]` is present. Use `?` for brevity, `map_err` to add context or transform.

### Tests still need no mocks

```rust
#[test]
fn increment_in_range_works() {
    let cfg = Config { initial_count: 0, max_increment: 100 };
    let svc = CounterService::new(&cfg);
    assert_eq!(increment(&svc, 5).unwrap(), 5);
}
```

Real service, no mocks, microseconds to run. Add traits/mocks only at the I/O boundary, when needed.

---

## 12. `main.rs` — Dissected

### `Cargo.toml` essentials

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[[bin]]
name = "mini_counter"
path = "src/main.rs"

[dependencies]
leptos = { version = "0.7", features = [] }
leptos_axum = { version = "0.7", optional = true }
axum = { version = "0.7", optional = true }
tokio = { version = "1", features = ["full"], optional = true }
thiserror = "1"

[features]
default = []
ssr = ["leptos/ssr", "leptos_axum", "axum", "tokio"]
hydrate = ["leptos/hydrate"]
```

### `crate-type = ["cdylib", "rlib"]`

- `rlib` → normal Rust library (server binary depends on this)
- `cdylib` → C-compatible dynamic library (what WASM produces)

Same source, different output formats.

### `optional = true` dance

`axum`, `tokio`, `leptos_axum` are server-only. They don't exist in WASM-land. Making them optional means they're only compiled when `ssr` feature is active. WASM build (`hydrate`) never sees them.

### `main.rs`

```rust
#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{routing::get, Router};
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use mini_counter::{app::App, config::Config, services::CounterService, state::AppState};

    // 1. Load config
    let config = Config::from_env().expect("failed to load config");

    // 2. Build services
    let counter = CounterService::new(&config);

    // 3. Build AppState
    let app_state = AppState::new(counter);

    // 4. Build Leptos config  
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;

    // 5. Generate routes from your Leptos app
    let routes = generate_route_list(App);

    // 6. Build Axum router
    let router = Router::new()
        .leptos_routes(&leptos_options, routes, App)
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(app_state);

    // 7. Bind and serve
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("listening on {addr}");
    axum::serve(listener, router).await.unwrap();
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use leptos::prelude::*;
    use mini_counter::app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
```

### `state.rs`

```rust
use std::sync::Arc;
use crate::services::CounterService;

#[derive(Clone)]
pub struct AppState {
    pub counter: Arc<CounterService>,
}

impl AppState {
    pub fn new(counter: CounterService) -> Self {
        Self { counter: Arc::new(counter) }
    }
}
```

### `#[cfg(feature = "ssr")]` on `main`

Compile-time conditional. The entire `main` is erased from WASM build. If it weren't, WASM would try to compile `tokio`, `axum`, sockets — none exist in browser sandbox.

The inverse: `hydrate()` is gated by `#[cfg(feature = "hydrate")]` — the WASM entry point. Server never compiles it.

**Same `main.rs` contains two entry points, each compiled only when appropriate. No runtime branching.**

### `#[tokio::main]`

Rust's actual `main` must be sync. Macro expands to:

```rust
fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { /* your async code */ });
}
```

Creates the tokio runtime, runs your async block, exits.

### Boot sequence ownership chain

```rust
let config = Config::from_env().expect("...");      // owned
let counter = CounterService::new(&config);         // borrows config
let app_state = AppState::new(counter);             // moves counter into Arc
```

Values created in sequence, each handing off, with `Arc` as the last holder.

### Leptos configuration

- `get_configuration(None)` reads `Leptos.toml` or defaults
- `generate_route_list(App)` SSR-renders App not for HTML, but to **discover all `<Route>` definitions**

### Three layers of router

```rust
Router::new()
    .leptos_routes(&leptos_options, routes, App)       // SSR + #[server] handlers
    .fallback(leptos_axum::file_and_error_handler(shell))  // WASM bundle, statics
    .with_state(app_state);                            // injects state
```

`.with_state(app_state)` stores AppState in Axum's TypeMap. Every handler asking for `State<AppState>` gets a clone (Arc clone, cheap).

### `hydrate()` entry point

```rust
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
```

- `#[wasm_bindgen]` exports the function so JS can call it
- `console_error_panic_hook::set_once()` redirects Rust panics to `console.error`
- `hydrate_body(App)` attaches event listeners and signals to existing server-rendered HTML — **without re-rendering**

Compare: `mount_to_body(App)` *replaces* server-rendered HTML — works but defeats SSR.

---

## 13. `server_fns.rs` — Dissected

```rust
use leptos::prelude::*;
use crate::server_logic::{self, LogicError};

#[cfg(feature = "ssr")]
use crate::state::AppState;

fn to_server_error(e: LogicError) -> ServerFnError {
    ServerFnError::ServerError(e.to_string())
}

#[server(GetCount, "/api")]
pub async fn get_count() -> Result<i32, ServerFnError> {
    let state = expect_context::<AppState>();
    server_logic::get_count(&state.counter)
        .map_err(to_server_error)
}

#[server(Increment, "/api")]
pub async fn increment(by: i32) -> Result<i32, ServerFnError> {
    let state = expect_context::<AppState>();
    server_logic::increment(&state.counter, by)
        .map_err(to_server_error)
}

#[server(Reset, "/api")]
pub async fn reset(target: i32) -> Result<i32, ServerFnError> {
    let state = expect_context::<AppState>();
    server_logic::reset(&state.counter, target)
        .map_err(to_server_error)
}
```

### What `#[server]` expands to

**Thing 1: A struct for the request type**

```rust
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Increment {
    pub by: i32,
}
```

Every argument becomes a field. This struct is what gets serialized into the HTTP request body on the client, deserialized on the server. **The function signature *is* the wire format.**

**Thing 2: A `ServerFn` trait impl, split by feature**

```rust
impl ServerFn for Increment {
    type Output = i32;
    fn url() -> &'static str { "/api/increment" }

    // SERVER side: real function body
    #[cfg(feature = "ssr")]
    async fn run_body(self) -> Result<i32, ServerFnError> {
        let by = self.by;
        let state = expect_context::<AppState>();
        server_logic::increment(&state.counter, by)
            .map_err(to_server_error)
    }

    // CLIENT side: HTTP stub
    #[cfg(not(feature = "ssr"))]
    async fn run_body(self) -> Result<i32, ServerFnError> {
        leptos::server_fn::client::send(self).await
    }
}
```

**This is the bridge. Same trait method, two bodies, selected at compile time.**

**Thing 3: Registration on the server**

During startup, Leptos maintains a global registry. `leptos_axum::generate_route_list` and `.leptos_routes(...)` walk this registry and mount each handler onto Axum.

That's why you never write `router.route("/api/increment", post(...))` manually.

### `expect_context::<AppState>()`

```rust
let state = expect_context::<AppState>();
```

The most important line in the file.

**What "context" is:** Leptos maintains a per-request `TypeMap` — `HashMap<TypeId, Box<dyn Any>>` keyed by type. Lives for one request.

**How AppState got there:** In `main.rs`, `.with_state(app_state)` gives Axum the state. `leptos_axum` middleware runs before every server fn:

```rust
let app_state: AppState = axum_state.clone();  // Arc clone, +1 refcount
provide_context(app_state);                     // puts it in TypeMap
```

By the time your fn body executes, `AppState` is already in context.

**Type parameter `<AppState>`:** turbofish. `TypeId` is unique per type at compile time — fast integer comparison, no string matching.

**Why `#[cfg(feature = "ssr")]` on the import:** `expect_context::<AppState>()` only runs in server body. Client stub never calls it. But the file is compiled for both. The `#[cfg]` guards the client from ever seeing `AppState`.

**`expect_context` vs `use_context`:** former returns `T` (panics if missing), latter returns `Option<T>`. AppState is never optional → `expect_context` is correct.

### `ServerFnError` and the error bridge

```rust
fn to_server_error(e: LogicError) -> ServerFnError {
    ServerFnError::ServerError(e.to_string())
}
```

**Why `ServerFnError` exists:** The client receives an HTTP response. It doesn't know about `LogicError` (server-only type behind `#[cfg]`). `ServerFnError` is the wire-safe error type, defined in Leptos, available on both sides, serializable.

**Information tradeoff:** `.to_string()` runs `Display` → produces the `#[error("...")]` string.
- `LogicError::OutOfRange { got: 0, max: 100 }` → `"increment must be between 1 and 100, got 0"`

You **lose** the enum variant (client can't `match` on `LogicError::OutOfRange`). You **gain** portability.

**For structured errors:** use `ServerFnError::WrappedServerError(T)` where `T: Serialize + Deserialize`. Define a shared `PublicError` type in a models crate compiled into both builds. (This is how `loan_machine_models/src/responses.rs` works in production.)

### Full error flow

```
server_logic::increment() → Err(LogicError::OutOfRange { got: 0, max: 100 })
        │
        ▼ map_err(to_server_error)
ServerFnError::ServerError("increment must be between 1 and 100, got 0")
        │
        ▼ serialized to JSON by Leptos
HTTP 500 { "error": "increment must be between 1 and 100, got 0" }
        │
        ▼ deserialized on client
Err(ServerFnError::ServerError("..."))
        │
        ▼ in component (via ErrorBoundary)
view! { <p>{msg}</p> }
```

### Why server functions are `async` even when logic isn't

- **Client side:** does network I/O — always async
- **Server side during SSR:** direct call, but `async fn` still needed for trait machinery to work uniformly. Sync work inside async fn resolves immediately.

### `#[server]` during SSR: short-circuit

```
Browser requests /counter
  → Axum SSR handler runs
    → Leptos renders <Counter/> on server
      → component calls get_count().await
        → are we on the server? yes, feature = "ssr"
          → run_body() calls real function directly
            → NO HTTP request made
          → returns Ok(42)
      → component renders <p>Count: 42</p>
    → full HTML returned
```

After hydration:

```
User clicks +1
  → component calls increment(1).await
    → are we on the client? yes, feature = "hydrate"
      → run_body() serializes {by: 1} → POST /api/increment
      → Axum receives, runs real function
      → response deserialized → Ok(43)
```

**Same call site. Different execution path. Compile-time selected.**

### What `.leptos_routes()` actually generates per server fn

```rust
router.route(
    "/api/increment",
    axum::routing::post(|
        State(app_state): State<AppState>,
        body: Bytes,
    | async move {
        provide_context(app_state);
        let args: Increment = deserialize(body)?;
        let result = args.run_body().await;
        serialize_response(result)
    })
)
```

You never write this. The macro wrote it.

---

## 14. `app.rs` — Component Layer Dissected

```rust
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::{Route, Router, Routes}, path};
use crate::server_fns::{get_count, increment, reset};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/mini_counter.css"/>
        <Title text="Mini Counter"/>
        <Router>
            <Routes fallback=|| view! { <p>"Not found"</p> }>
                <Route path=path!("/") view=Counter/>
            </Routes>
        </Router>
    }
}

#[component]
pub fn Counter() -> impl IntoView {
    // 1. Resource: SSR-aware data fetching
    let count_resource = Resource::new(
        || (),
        |_| async move { get_count().await },
    );

    // 2. Actions: mutations
    let increment_action = Action::new(|by: &i32| {
        let by = *by;
        async move { increment(by).await }
    });

    let reset_action = Action::new(|target: &i32| {
        let target = *target;
        async move { reset(target).await }
    });

    // 3. Derived signal: current display value
    let count = move || {
        if let Some(Ok(n)) = increment_action.value().get() {
            return Some(n);
        }
        if let Some(Ok(n)) = reset_action.value().get() {
            return Some(n);
        }
        count_resource.get().and_then(|r| r.ok())
    };

    // 4. Pending signals
    let incrementing = increment_action.pending();
    let resetting = reset_action.pending();
    let loading = move || count_resource.loading().get();

    view! {
        <div class="counter">
            <h1>"Counter"</h1>

            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                <p class="count">
                    {move || count().map(|n| n.to_string()).unwrap_or_default()}
                </p>
            </Suspense>

            <ErrorBoundary fallback=|errors| view! {
                <div class="error">
                    <For
                        each=move || errors.get()
                        key=|(id, _)| *id
                        children=|(_, e)| view! { <p>{e.to_string()}</p> }
                    />
                </div>
            }>
                {move || increment_action.value().get().map(|r| r.map(|_| ()))}
            </ErrorBoundary>

            <div class="controls">
                <button
                    on:click=move |_| { increment_action.dispatch(1); }
                    disabled=move || incrementing.get() || loading()
                >
                    {move || if incrementing.get() { "..." } else { "+1" }}
                </button>

                <button
                    on:click=move |_| { increment_action.dispatch(10); }
                    disabled=move || incrementing.get() || loading()
                >
                    "+10"
                </button>

                <button
                    on:click=move |_| { reset_action.dispatch(0); }
                    disabled=move || resetting.get() || loading()
                >
                    {move || if resetting.get() { "Resetting..." } else { "Reset" }}
                </button>
            </div>
        </div>
    }
}
```

### Component body runs exactly once

**Most important mental model shift from React.**

In React: component function re-runs on every state change. Body is the "render function." Hooks survive repeated calls.

In Leptos: component body runs **once** — at creation/mount. Never again. What you return is not HTML but a **graph of reactive closures**. Those closures re-run individually when their signal dependencies change.

```rust
#[component]
pub fn Counter() -> impl IntoView {
    let (x, set_x) = signal(0);   // ← runs once

    view! {
        <p>{move || x.get() * 2}</p>   // ← this closure re-runs when x changes
    }
}
```

The closure is a reactive node in the dependency graph. When `x` changes, only that closure re-runs, only that text node updates. The `<p>` doesn't re-render. The component fn doesn't re-run.

This is why you see `move ||` everywhere. Those aren't callbacks — they're **reactive units**.

### `signal` — the primitive

```rust
let (count, set_count) = signal(0_i32);
```

Returns split handle: `(ReadSignal<T>, WriteSignal<T>)`. Type system enforces unidirectional flow — give children `ReadSignal`, keep `WriteSignal`.

For:
- **Local sync state** → `signal`
- **Async server data** → `Resource`
- **Server mutations** → `Action`

### Signals are `Copy`

```rust
let (count, set_count) = signal(0);
let double = move || count.get() * 2;   // count copied into closure
let triple = move || count.get() * 3;   // count copied again, same signal
```

Both closures read the same signal. No `Arc`, no clones. The handles are small (an index into a runtime arena), not the data itself.

### `Resource` — async signal

```rust
let count_resource = Resource::new(
    || (),                                      // source signal (key)
    |_| async move { get_count().await },       // async loader
);
```

- **Source signal** `|| ()`: closure returning the "key" the resource depends on. When this changes, the resource re-fetches. `|| ()` = no dependencies. `move || user_id.get()` = refetches when `user_id` changes.
- **Async loader**: the actual fetch.

### What Resource gives you

```rust
count_resource.get()        // Option<Result<i32, ServerFnError>>
count_resource.loading()    // Signal<bool>
```

`.get()` returns `None` while loading, `Some(Ok(n))` on success, `Some(Err(e))` on failure. Full state machine in one type.

### SSR behavior of Resource

During SSR, Resource calls the async loader on the server — direct fn call. Result gets **serialized into the HTML as inline `<script>` data**. On the client, Resource reads the inlined data instead of making another HTTP call. **No double-fetch.**

This is **resource dehydration/hydration** — server bakes data into HTML, client picks it up.

### `<Suspense>` — async boundary

```rust
<Suspense fallback=move || view! { <p>"Loading..."</p> }>
    <p>{move || count()...}</p>
</Suspense>
```

Watches for unresolved Resources in children. Renders fallback while loading; renders children when resolved.

On server: streams HTML — fallback first, then content swap via tiny `<script>`. No blocking.

**Always wrap Resource reads in Suspense.**

### `Action` — async mutations

```rust
let increment_action = Action::new(|by: &i32| {
    let by = *by;
    async move { increment(by).await }
});
```

Three properties distinguishing Action from Resource:

1. **Doesn't auto-run.** Trigger explicitly with `dispatch(1)`. Maps to user gestures.
2. **Has pending state.** `action.pending()` is `Signal<bool>`.
3. **Stores last result.** `action.value()` is `Signal<Option<Result<T, E>>>`.

### Why argument is `&i32` and you copy

```rust
Action::new(|by: &i32| {
    let by = *by;                           // copy out
    async move { increment(by).await }      // move copy into async block
})
```

`Action::new` passes a reference. Standard pattern: copy/clone what you need, then `async move`.

### `dispatch` vs `await`

```rust
on:click=move |_| { increment_action.dispatch(1); }
```

`dispatch` is **synchronous**. Schedules async work, returns immediately. UI stays responsive. Action's signals update reactively as async progresses.

**Never `.await` an action from an event handler.** Event handlers are sync closures. `dispatch` is the bridge.

### Derived signals — composing signals

```rust
let count = move || {
    if let Some(Ok(n)) = increment_action.value().get() { return Some(n); }
    if let Some(Ok(n)) = reset_action.value().get() { return Some(n); }
    count_resource.get().and_then(|r| r.ok())
};
```

Not a `signal()`. Just a closure — a **derived signal** (or "memo" if wrapped in `Memo::new`). Inside reactive context, subscribes to every signal it reads.

When any source changes, closure re-runs, DOM updates.

### Optimistic priority logic

When `increment_action` fires:
1. `pending()` → true → button shows "..."
2. HTTP call
3. `value()` → `Some(Ok(43))`
4. `count` closure now returns 43 — from action, not resource
5. Display updates immediately

Action result takes priority over stale resource value. **UI reflects what user just did, not stale fetched value.**

For full reconciliation, invalidate the resource:

```rust
Effect::new(move |_| {
    if increment_action.value().get().is_some() {
        count_resource.refetch();
    }
});
```

### `view!` macro — what it produces

JSX-like syntax compiled into Rust code constructing DOM nodes (client) or HTML strings (server). Not a runtime template — actual code at compile time.

**`on:click`:** compiles to `element.add_event_listener("click", closure)` on client. Ignored during SSR. Closure captures by copy (Action is Copy).

**`disabled=move || ...`:** any attribute set to a closure becomes a reactive binding. When signal changes, attribute updates **in-place**. No re-render of the button — just `setAttribute`/`removeAttribute`.

**`{move || ...}`:** closure returning a value inside braces is a reactive text node. When signal changes, only that text node updates.

**Compare to React:** state change → entire component rerenders → VDOM diff figures out only attribute changed. **Leptos skips the diff** — reactive system already knows exactly which DOM node to update.

### `<ErrorBoundary>`

```rust
<ErrorBoundary fallback=|errors| view! {
    <For each=move || errors.get()
         key=|(id, _)| *id
         children=|(_, e)| view! { <p>{e.to_string()}</p> }/>
}>
    {move || increment_action.value().get().map(|r| r.map(|_| ()))}
</ErrorBoundary>
```

Catches `Err` values propagating through children. `errors: ArcRwSignal<Errors>` — reactive map of current errors.

`map(|r| r.map(|_| ()))`: convert `Result<i32, _>` to `Result<(), _>`. We don't want to display the `Ok` value here (that's the count display's job). Mapping `Ok` to `()` drops the value while keeping error path intact.

### `<For>` — keyed list rendering

```rust
<For each=...  key=|(id, _)| *id  children=|(_, e)| ... />
```

- `each` → closure returning iterable
- `key` → unique stable key per item (like React's `key`)
- `children` → render closure

Items with matching keys: existing DOM nodes updated. New keys: new nodes. Removed keys: destroyed. **Avoids re-rendering the whole list when one item changes.**

### The reactive graph

```
count_resource ──────────────────────┐
                                     ▼
increment_action.value() ──► count() closure ──► <p> text node
                                     ▲
reset_action.value() ────────────────┘

increment_action.pending() ──► <button disabled>
                           ──► <button> text node ("..." / "+1")

reset_action.pending() ──► <reset button disabled>
                       ──► <reset button> text node

count_resource.loading() ──► all <button disabled>
```

Every arrow is a subscription. Every node on the right is a minimal DOM operation. When `pending()` flips true: exactly two DOM operations happen. **Surgical updates.**

This is the payoff of "component runs once": dependency graph built once, updates flow through it surgically.

---

## 15. The Full Lifecycle

### SSR through interaction

```
SERVER (SSR)
  App() runs once
  Counter() runs once
    Resource::new() → calls get_count() directly → Ok(42)
    Action::new() → creates action handles (no-ops on server)
    view! → compiles reactive graph → renders to HTML string
  HTML emitted:
    <p class="count">42</p>
    <button>+1</button>
    ... + inline <script>{"count_resource": 42}</script>

BROWSER (hydration)
  WASM loads, hydrate() called
  Counter() runs once
    Resource::new() → reads inlined data (42), no HTTP call
    Action::new() → creates real action handles
    view! → attaches to existing DOM nodes, wires event listeners
  UI is live

USER CLICKS "+1"
  on:click fires
  dispatch(1) called
  increment_action.pending() → true
    → button shows "..." (text node update)
    → buttons disabled (attribute update)
  POST /api/increment {by: 1} → server
    → mutex lock → 42+1=43 → mutex release
    → HTTP 200 {value: 43}
  increment_action.value() → Some(Ok(43))
    → count() re-evaluates → 43
    → <p> text updates to "43"
  increment_action.pending() → false
    → button shows "+1"
    → buttons enabled
```

### End-to-end call path

```
BROWSER
  User clicks "+1"
  component calls increment(1)
  [hydrate build] serialize {by: 1}
  POST /api/increment  ──────────────────────────────────────┐
                                                             │
SERVER                                                       ▼
  Axum receives POST /api/increment                   TCP listener
  middleware: provide_context(app_state.clone())      Axum router
  deserialize body → Increment { by: 1 }              route match
  Increment::run_body() called                             │
  ↓                                                        │
  expect_context::<AppState>()   ← pulls Arc from context  │
  server_logic::increment(&state.counter, 1)               │
  ↓                                                        │
  validate: 1 is in [1..100] ✓                             │
  counter.get() → lock mutex → read → release              │
  1 + 1 = 2, no overflow ✓                                 │
  counter.add(1) → lock mutex → write 2 → release          │
  Ok(2)                                                     │
  ↓                                                        │
  serialize Ok(2) → HTTP 200 {"value": 2}  ────────────────┘
  
BROWSER
  deserialize → Ok(2)
  Action value signal updates to Ok(2)
  component re-renders "Count: 2"
```

---

## Key Principles Summary

1. **Owned + `Clone` is the default.** `Arc` is for genuine cross-boundary sharing or expensive clones.
2. **`Arc<Mutex<T>>` is the canonical shared-mutable pattern.** Outer Arc for sharing, inner Mutex for mutation.
3. **Inner Arcs in services**, not outer. Lock granularity matters; immutable fields shouldn't share a lock.
4. **Never hold `std::sync::Mutex` across `.await`.** Use `tokio::sync::Mutex` if you must.
5. **Method return values should be owned**, especially from behind a lock.
6. **References for sync, scoped work; owned/Arc across `.await`.**
7. **Each layer has its own error type.** Translate at boundaries.
8. **Take the least powerful thing that works:** `&T` over `Arc<T>` over owned `T`.
9. **Validate before mutate.** Never put state in a bad spot then error out.
10. **Component body runs once.** Closures inside `view!` are the reactive units.
11. **Signals are `Copy`** — capture by value, no clone needed.
12. **Resource for fetches, Action for mutations, signal for local state.**
13. **`#[server]` is the bridge** — same call site, two bodies, compile-time selected.
14. **`expect_context::<AppState>()`** is how server fns access state — context is per-request TypeMap.
15. **Wrap Resources in `<Suspense>`**, errors in `<ErrorBoundary>`.
16. **Same source compiles to two binaries** — server (native) and client (WASM). `#[cfg]` guards.

---

## Architecture Mapping to Production

| Mini project | Production (`loan_machine`) |
|---|---|
| `CounterService` | `CoopRegistry`, `Provider`, `PrivyClient` |
| `Arc<Mutex<i32>>` | `Arc<Provider>`, `Arc<PrivyClient>` (Arc for sharing, not mutation) |
| `server_logic::increment` | `server_logic::create_coop`, `server_logic::vinculation` |
| `#[server] increment` | `#[server] create_coop`, `#[server] vinculate` |
| `LogicError` translation | Domain errors → `ServerFnError::WrappedServerError` with shared models |
| Single `<Counter/>` | `<CreateCoop/>`, `<Cooperatives/>`, `<Vinculation/>` |

The full loop in production: config flows into services, services live in AppState behind Arc, AppState gets injected into Leptos context, server functions pull it via `expect_context`, server functions are called by components via `Action` and `Resource`, reactive signals drive surgical DOM updates.

**This is the complete mental model.** Every pattern in the production project is this same loop with more fields in AppState and more steps in the logic layer.
