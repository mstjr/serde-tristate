# serde-tristate

Three-state field type for HTTP PATCH request bodies, with automatic serde integration.

## The problem

HTTP PATCH semantics require distinguishing three states for a field:

| State | JSON | Meaning |
|---|---|---|
| `Value(T)` | `"name": "Alice"` | Set field to this value |
| `None` | `"name": null` | Clear the field |
| `Undefined` | *(absent)* | Leave the field unchanged |

`Option<T>` only covers two of these. `Tristate<T>` covers all three.

## Why not `Option<Option<T>>`?

`Option<Option<T>>` can technically encode three states, but falls apart with serde:

```rust
// Option<Option<T>> - broken with serde
struct UpdateUser {
    bio: Option<Option<String>>,
}

// `None` and `Some(None)` both deserialize from JSON `null` - indistinguishable
// serde cannot tell "field absent" from "field null" without manual impls
serde_json::from_str::<UpdateUser>(r#"{}"#)         // bio → None ✓
serde_json::from_str::<UpdateUser>(r#"{"bio":null}"#) // bio → None ✗ (want Some(None))
```

serde collapses `null` → `None` at the outermost layer, making `Some(None)` unreachable via derive.

`Tristate<T>` solves this by deserializing from `Option<T>` (null → `Tristate::None`) and using `#[serde(default)]` for the absent case (missing key → `Tristate::Undefined`):

```rust
// Tristate<T> - correct
serde_json::from_str::<UpdateUser>(r#"{}"#)              // bio → Tristate::Undefined ✓
serde_json::from_str::<UpdateUser>(r#"{"bio":null}"#)    // bio → Tristate::None      ✓
serde_json::from_str::<UpdateUser>(r#"{"bio":"hello"}"#) // bio → Tristate::Value(..) ✓
```

`Option<Option<T>>` also serializes `Some(None)` as `null` and `None` as absent - which requires `#[serde(skip_serializing_if = "Option::is_none")]` and still conflates the two null states on the deserialize side.

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
serde-tristate = "0.1"
serde = { version = "1", features = ["derive"] }
```

Annotate your struct with `#[serde_tristate]` and derive `Serialize`/`Deserialize` normally - no per-field annotations needed:

```rust
use serde_tristate::{Tristate, serde_tristate};
use serde::{Serialize, Deserialize};

#[serde_tristate]
#[derive(Serialize, Deserialize)]
struct UpdateUser {
    name: Tristate<String>,
    age:  Tristate<u32>,
    bio:  Tristate<String>,
}
```

### Serialization

```rust
let patch = UpdateUser {
    name: Tristate::Value("Alice".into()),
    age:  Tristate::None,       // → null
    bio:  Tristate::Undefined,  // → skipped
};

serde_json::to_string(&patch)?;
// {"name":"Alice","age":null}
```

### Deserialization

```rust
// absent field → Tristate::Undefined
// null field   → Tristate::None
// any value    → Tristate::Value(v)
let patch: UpdateUser = serde_json::from_str(r#"{"name":"Bob"}"#)?;
// patch.name → Tristate::Value("Bob")
// patch.age  → Tristate::Undefined
// patch.bio  → Tristate::Undefined
```

### Conversions

```rust
let p: Tristate<i32> = 42.into();               // Tristate::Value(42)
let p: Tristate<i32> = Some(42).into();         // Tristate::Value(42)
let p: Tristate<i32> = Option::<i32>::None.into(); // Tristate::None

let opt: Option<Option<i32>> = p.into(); // Undefined→None, None→Some(None), Value(v)→Some(Some(v))
```

### Combinators

```rust
patch.name.map(|s| s.to_uppercase());
patch.age.unwrap_or(0);
patch.bio.and_then(|s| if s.is_empty() { Tristate::None } else { Tristate::Value(s) });
```

### Applying to an existing entity

```rust
let mut user = get_user_from_db();

patch.name.apply_to_tristate(&mut user.name);  // required field
patch.bio.apply_to_option(&mut user.bio);      // Option<T> field
```

### Enums

`#[serde_tristate]` works on enums with named variant fields too:

```rust
#[serde_tristate]
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
enum UpdateEvent {
    User { name: Tristate<String>, age: Tristate<u32> },
    Other,
}
```
