# Rust Errors and the Type-as-Shape Philosophy

A consolidated reference distilled from refactoring the `loan-machine` services
(`subgraph`, `privy_auth`, `coop_deployment`, `coop_registry`, `server_logic`).

The whole document hangs on one idea:

> **The shape of the program is in the types, and the types are checked by the
> compiler.**

Errors are the cleanest place to see this principle at work. Most of what
follows is the error story; the last sections zoom out to the philosophy that
the error story is one instance of.

---

## Part I — Errors are Values

In Rust, errors are not exceptions. They don't unwind the stack, they don't
get caught somewhere up the call chain, they don't appear out of nowhere in
production logs. Every fallible function declares its failure modes in its
return type:

```rust
fn verify(&self, token: &str) -> Result<PrivyClaims, PrivyAuthError>
```

`PrivyAuthError` is a normal Rust enum. The function either returns
`Ok(claims)` or `Err(PrivyAuthError::Variant(...))`. The compiler refuses to
compile code that ignores the `Err` case — you must either handle it or
propagate it.

That's the foundation. Errors are values. They flow through your program like
any other data.

---

## Part II — The `Error` trait and the source chain

Rust's `std::error::Error` trait has one method that matters here:

```rust
fn source(&self) -> Option<&(dyn Error + 'static)>
```

This lets errors *link*. Your high-level error wraps a mid-level error wraps a
low-level error wraps an OS error. `source()` walks the chain, one link at a
time. Tools like `tracing`, `anyhow`, and structured loggers print the whole
chain when you format with the `{:#}` flag.

A real chain looks like:

```
VinculationLogicError::ServerLogicBlockchain(
    BlockchainError::ContractRevert {
        message: "Esta cooperativa não está ativa.",
        source: alloy::contract::Error::TransportError(
            RpcError(
                reqwest::Error(
                    io::Error("Connection refused")
                )
            )
        )
    }
)
```

Six layers, each one a link in the chain. Top-level Display gives you the
Portuguese message; walking `source()` gives you the full diagnostic.

**The empty-impl trap.** A naive `impl std::error::Error for MyError {}` makes
`source()` return `None` for every variant — even if those variants wrap inner
errors. That kills the chain at your layer. Anyone walking the chain above you
sees nothing below your error. This was the bug in every service before we
switched to `thiserror`.

---

## Part III — `thiserror`, `#[from]`, `#[source]`

`thiserror` is a derive macro that generates the `Display` and `Error` impls
mechanically. The two attributes we used most:

### `#[from]`

Generates **two** things:

1. The `source()` link (returns the inner error).
2. An `impl From<InnerError> for OuterError` that wraps `InnerError` into the
   variant.

That second piece is what makes `?` work. The `?` operator desugars to
`Err(From::from(e))`, so without a `From` impl, `?` won't compile. With
`#[from]`, every line that returns `InnerError` can use a bare `?` and the
conversion happens automatically.

```rust
#[derive(Debug, thiserror::Error)]
pub enum BlockchainError {
    #[error("contract call failed")]
    Call(#[from] alloy::contract::Error),
}

// Now this compiles:
let result = registry.coops(coop_id).call().await?;
```

### `#[source]`

Generates **only** the `source()` link. No `From` impl. So `?` won't pick this
variant up automatically; you have to `map_err` explicitly.

```rust
#[error("gas estimation failed")]
GasEstimate(#[source] alloy::contract::Error),

// Explicit:
.estimate_gas().await.map_err(BlockchainError::GasEstimate)?;
```

### Why both exist

The constraint: **only one `#[from]` per source type**. If two variants both
say `#[from] alloy::contract::Error`, the compiler can't decide which variant
the conversion should produce. Ambiguous. Banned.

So when two variants need to wrap the same source type — `Call` and
`GasEstimate` both wrap `alloy::contract::Error` — one gets `#[from]` (the
default for `?`), the other gets `#[source]` (explicit `map_err`). You're
encoding a deliberate choice: "the common path is `Call`; tag the rare one as
`GasEstimate` by hand."

### `#[error(transparent)]`

Makes the variant's Display delegate entirely to its source. No prefix, no
wrapping. Use it when you're just bubbling an inner error up without adding
context:

```rust
#[error(transparent)]
ServerLogicBlockchain(#[from] BlockchainError),
```

Format this and you get *exactly* what `BlockchainError`'s Display would
produce. `format!("{err}")` in the route handler returns
`"Esta cooperativa não está ativa."` even though the value is wrapped two
layers deep. The Display walks down to the deepest meaningful message.

### Constructor-as-function shorthand

A tuple-struct enum variant is itself a function:

```rust
BlockchainError::GasEstimate     // type: fn(alloy::contract::Error) -> BlockchainError
```

So these are equivalent:

```rust
.map_err(|e| BlockchainError::GasEstimate(e))
.map_err(BlockchainError::GasEstimate)
```

Both compile to the same code. Pick the form you find clearer.

---

## Part IV — The `to_string()` Trap

`e.to_string()` is `format!("{e}")` under the hood — it asks the error for its
human-readable Display rendering and copies that into a fresh `String`.

The cost: errors carry **two layers** of information.

1. A **type** with structured variants and fields you can `match` on.
2. A **Display rendering** for humans.

`to_string()` keeps the second and discards the first. After:

```rust
.map_err(|e| MyError::Variant(e.to_string()))
```

— the original typed error is gone forever. The closure consumed it, rendered
it to text, dropped the original. `MyError::Variant(String)` has no `source()`
to walk. Anything you might want to do programmatically — retry transient
failures, surface specific UI for known revert types, distinguish "RPC node
hiccup" from "contract reverted" — is now impossible. Just text.

### When stringifying is wrong

- **Inside service boundaries.** Callers may want to branch on the error
  variant. Once you've stringified, they can't.
- **In intermediate layers** that other code wraps further. You're a chain
  link; flattening here means everyone above you also flattens by default.
- **When the inner error has structured fields.** JSON-RPC errors carry
  numeric codes (`-32003 = nonce too low`). HTTP errors carry status codes.
  Solidity reverts carry 4-byte selectors. These are *data*, not just
  messages.

### When stringifying is right

- **Crossing a serialization boundary.** JSON response, log line, database
  column — anywhere you're leaving the Rust type system. At that point you've
  extracted everything you'll ever extract.
- **The inner type doesn't satisfy your bounds.** `#[from]` requires
  `Send + Sync + 'static`. Some errors don't fit. `String` fits everything.
- **Display is genuinely all that matters.** A CPF format failure, a config
  parse error — nobody is going to programmatically branch on the message.
- **Collapsing redundant variants.** Five sources that all mean "the network
  is unhappy" and you never distinguish them — one stringified `Network`
  variant is fine.

### Rule of thumb

> **Stringify at the exit, not the entrance.**

The closer you are to logs / HTTP responses / human display, the more
legitimate `to_string()` is. The deeper you are in the call stack, the more
each `to_string()` costs above you in flexibility. In services, keep types
typed; render to text once at the route handler.

---

## Part V — The Patterns

### Pattern 1: Wrap, don't stringify

```rust
// Wrong — flattens to String, kills source chain
.map_err(|e| BlockchainError::Network(e.to_string()))?

// Right — wraps the typed error, source preserved
.map_err(BlockchainError::from_call)?
// or with #[from]:
?
```

### Pattern 2: One variant per error *family*

A "family" means "the kind of failure that's structurally possible at this
step." Different alloy operations return different error types:

| Operation | Error type | Family |
|---|---|---|
| `.call().await` | `alloy::contract::Error` | Contract call (may include Solidity revert) |
| `.send().await` | `alloy::contract::Error` | Same |
| `.get_code_at(...)` | `alloy::transports::TransportError` | Pure RPC, no revert possible |
| `pending.watch()` | `alloy::providers::PendingTransactionError` | Tx confirmation |
| `.estimate_gas()` | `alloy::contract::Error` | Same as call |

Three families, three variants:

```rust
#[derive(Debug, thiserror::Error)]
pub enum CoopDeploymentError {
    #[error("contract call failed")]
    Call(#[from] alloy::contract::Error),

    #[error("transport error")]
    Transport(#[from] alloy::transports::TransportError),

    #[error("transaction confirmation failed")]
    PendingTx(#[from] alloy::providers::PendingTransactionError),
}
```

Each `#[from]` is unambiguous because the source types are distinct. Every
fallible call site uses a bare `?`. The compiler picks the right variant by
looking at the type being propagated.

### Pattern 3: The classifier

When the wrapping decision depends on *inspecting* the error (e.g., "is this a
Solidity revert with a known selector?"), you can't use `#[from]` — `?` would
unconditionally pick one variant. You need a function:

```rust
impl BlockchainError {
    pub fn from_call(err: alloy::contract::Error) -> Self {
        if let Some(bytes) = extract_revert_data(&err) {
            Self::ContractRevert {
                message: translate_revert(&bytes),
                source: err,
            }
        } else {
            Self::Call(err)
        }
    }
}
```

Note that `from_call` *moves* the original `err` into one of the two variants
— **both branches preserve it** in the `source` field. The friendly message
is added *alongside* the typed error, not instead of it.

Call sites use `.map_err(BlockchainError::from_call)?` instead of `?`. The
`Call` variant uses `#[source]` (not `#[from]`) so `?` doesn't accidentally
skip the classifier.

### Pattern 4: `transparent` for pass-through wrapping

When the outer layer adds *no information* — it just declares "this came from
the layer below" — use `transparent`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum VinculationLogicError {
    #[error("Endereço de carteira inválido")]
    ServerLogicInvalidWalletAddress,

    #[error(transparent)]
    ServerLogicBlockchain(#[from] BlockchainError),
}
```

The validation variants (`InvalidWalletAddress`, etc.) have their own
messages. The pass-through variants delegate to the source. Display the
top-level error and you get the deepest meaningful message —
`"Esta cooperativa não está ativa."` instead of
`"server logic error: blockchain error: contract revert: Esta cooperativa não está ativa."`.

### Pattern 5: Rich variant for translated errors

The two-field variant carries both the translated message and the typed source:

```rust
#[error("{message}")]
ContractRevert {
    message: String,
    #[source]
    source: alloy::contract::Error,
}
```

- `Display` returns the translated message — exactly what you want at the UI
  edge.
- `source()` returns the alloy error — exactly what you want for logs and
  retry logic.

One value. Two consumers. No duplication.

---

## Part VI — Layering

The architectural rule that ties all the patterns together:

```
Service layer       → typed errors, full source chains, no stringification
       ↓ #[from]
server_logic layer  → typed errors that wrap service errors (transparent or
                      with-context), plus its own validation variants
       ↓ ?
Route handler /     → THIS is where you stringify. Convert to ServerFnError
ServerFn boundary     or JSON response. Log the chain. Render Display.
       ↓ String
UI                  → string only. Display in toast / form error / etc.
```

Inside services, every `?` propagates a typed error. Inside `server_logic`,
every `?` propagates a typed error. The transition to `String` happens
**once**, at the boundary, in the handler.

Where `friendly_from_error` legitimately lives:

```rust
// In a route handler, top of the call stack:
match service.do_thing().await {
    Ok(value) => Ok(value),
    Err(err) => {
        tracing::error!("{:#}", err);                 // full chain to logs
        Err(ServerFnError::ServerError(format!("{err}")))  // top message to UI
    }
}
```

Inside services, `friendly_from_error` is a code smell. At the boundary, it's
the right tool.

---

## Part VII — The Deeper Philosophy

Errors are one instance of a broader principle. To see the principle, step
back from errors and look at types in general.

### Three layers of type strictness

**Layer 1 — Types describe shape.** `int`, `String`, `User`, `Vec<Customer>`.
Most languages do this much. The shape tells you what fields and variants
exist; it helps the human reader and powers autocomplete.

**Layer 2 — Types describe what's *legal*.** `Option<User>` says "this is
either a User or nothing, and you cannot pretend the nothing case doesn't
exist." `Result<T, E>` says "this might fail, and the failure has a specific
type." The compiler enforces exhaustive handling. Compare to Java where `null`
is silently a member of every reference type.

The slogan: **"make illegal states unrepresentable."** If a logged-out user
shouldn't be able to access billing, encode it: `User<LoggedIn>` and
`User<LoggedOut>` are different types, the `billing()` method exists only on
the first, and the compiler refuses to call it on the second. The bug becomes
a compile error instead of a forgotten runtime check.

**Layer 3 — Types describe behavior, scope, capability.** Rust pulls ahead
here:

- `&mut T` — "I have exclusive write access for the scope of this borrow."
  Behavior contract.
- `'a` lifetimes — "this reference is valid at least as long as this scope."
  Temporal claim.
- `T: Send + Sync` — "this type is safe across thread boundaries." Capability
  claim.
- `T: Hash + Eq` on a generic — "I'll accept any T that supports being a
  hashmap key." Protocol claim.

Each is a property the compiler verifies before your code runs. The runtime
never has to check "is this thread-safe, is this borrow valid, did I forget
the None case" — the type system already proved it.

### What this means in practice

In most languages, a function signature tells you roughly what data flows in
and out. In Rust, a function signature tells you:

- What data flows.
- Who owns it.
- How long the borrows live.
- What threads it can cross.
- What protocols its types support.
- What failure modes you must handle.

The signature is a contract. The compiler is a contract enforcer. Reading a
Rust file is reading a sequence of proofs; if it compiles, those proofs hold.

### The error story is one instance of this

`Result<T, BlockchainError>` says "this can fail, and the failure modes are
exactly the variants of `BlockchainError`." Match exhaustively or don't match
at all — the compiler decides. Stringify at the wrong layer and you erase
information that callers above you might have wanted; the compiler catches
the *type narrowing* directly. The same instinct that gave Rust its borrow
checker gives it a useful error story: encode the shape, let the compiler
prove it.

---

## Part VIII — The Tradeoffs

Rust isn't the apex of language design. It's a particular tradeoff that says
"compile-time correctness matters enough to be worth the friction." Other
languages chose differently, and for some problems they're correct.

| Language | What it optimizes for | What it gives up |
|---|---|---|
| Python | Iteration speed, ecosystem reach, dynamism | Compile-time guarantees |
| Ruby | Malleability, monkey-patching, DSL fluency | Predictability |
| JavaScript | Universal runtime, async-first | Type discipline (until TS) |
| Java | Familiarity at scale, JVM ecosystem | Concise types, true sum types |
| Go | Teachability, fast compiles | Generics (until 1.18), null safety |
| Haskell | Mathematical purity, total functions | Pragmatic ecosystem, ease of FFI |
| C | Raw control without ceremony | Memory safety, ownership model |
| Rust | Compile-time correctness, zero-cost abstractions | Iteration speed, dynamism |

Picking a language is picking a tradeoff curve. A scientist exploring data in
pandas and a kernel team writing a hypervisor in Rust aren't disagreeing
about which language is better — they're correctly fitting their tools to
their problems.

What you're paying for in Rust:

1. **Slower iteration.** `cargo check` is seconds. Real builds are minutes.
   Hot-reloading is harder. Notebook-style exploration is unnatural.
2. **Cognitive load up front.** Borrow checker, lifetimes, generics, trait
   bounds. The first six months are friction.
3. **Less dynamism.** No monkey-patching, no runtime class generation, no
   adding methods to existing types. Plugins and DI containers are clumsier.
4. **Smaller ecosystem in some domains.** Especially data science, ML
   research, and rapid web prototyping.

What you're getting:

1. **Memory safety without GC.** Performance of C with safety of GC'd
   languages. Whole bug categories impossible.
2. **Fearless concurrency.** Send/Sync makes data races impossible at compile
   time. Async/await without callback hell.
3. **Refactoring confidence.** Change a type, follow the compiler errors,
   ship. The shape of the program is enforced.
4. **Documentation in types.** Function signatures tell you what's possible.
   You don't need to read the body to know what can fail.

If you're doing systems work, services that need to be reliable, or anything
where correctness compounds — Rust's tradeoff makes sense. If you're
exploring a problem and need to iterate fast, or building something throwaway,
or shipping to a team that values onboarding speed — pick a different
tradeoff.

---

## Quick Reference

### Error enum template

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyError {
    // Self-contained variant: just a message
    #[error("invalid input: {0}")]
    Invalid(String),

    // Wrap a foreign error, generate `From` impl, enable `?`
    #[error("network error")]
    Network(#[from] reqwest::Error),

    // Wrap a foreign error, source-only (when type is already #[from] elsewhere)
    #[error("gas estimation failed")]
    GasEstimate(#[source] alloy::contract::Error),

    // Pass through an inner error verbatim
    #[error(transparent)]
    Inner(#[from] InnerError),

    // Rich variant: translated message + typed source
    #[error("{message}")]
    Classified {
        message: String,
        #[source]
        source: alloy::contract::Error,
    },
}
```

### Decision flowchart

> **Question:** I have a fallible operation. How do I propagate the error?

```
Is the inner error type already wrapped by a #[from] variant?
├─ No  → Add a new variant with #[from]. Use bare `?`.
└─ Yes → Do you need to classify (e.g. extract revert)?
         ├─ No  → Use #[from] on the existing variant. Bare `?`.
         └─ Yes → Write a classifier (`from_xxx`). Use #[source].
                  Call sites: `.map_err(MyError::from_xxx)?`
```

> **Question:** Should I stringify here?

```
Am I at a boundary that leaves the type system?
(JSON response, log line, UI string, database column)
├─ Yes → Stringify is correct.
└─ No  → Wrap with the typed error. Don't stringify.
```

### Smell guide

| Code smell | What's wrong | Fix |
|---|---|---|
| `.map_err(\|e\| MyError::X(e.to_string()))` | Throws away typed source | `.map_err(MyError::X)` with `#[from]` or `#[source]` on `X(InnerError)` |
| `impl Error for MyError {}` (empty) | `source()` returns None | Use `thiserror`, add `#[source]` or `#[from]` to wrapping variants |
| Same `String`-only variant catching 5 different errors | No structural inspection possible | Split into per-family variants or use a classifier |
| `friendly_from_error(&e)` inside a service | Stringifies at the wrong layer | Move to the route handler boundary |
| `#[error("...")]` with manual format prefixing transparent variants | Display includes redundant prefixes | Use `#[error(transparent)]` for pass-through variants |

---

## Closing thought

The error story is small. The principle is large.

Every time you encode an invariant in the type system, the compiler becomes a
collaborator in maintaining it. Every time you stringify, narrow, or erase a
type, you trade compile-time enforcement for runtime hope. The choice is local
but the consequences propagate — narrowed types stay narrowed, lost
information stays lost, runtime hope stays runtime hope.

Pick the right place to encode each invariant. Pick the right place to render
to text. The rest is type-driven mechanics, and the compiler will help you
keep it correct.
