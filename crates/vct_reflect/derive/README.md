# vct_reflect proc-macro

## `#[derive(Reflect)]`

```rust, ignore
#[derive(Reflect)]
struct Foo { /* ... */ }
```

`#[derive(Reflect)]` implements:

- `TypePath`
- `Typed`
- `Reflect`
- `GetTypeTraits`
- `FromReflect`
- `Struct` for `struct T { ... }`
- `TupleStruct` for `struct T(...);`
- `Enum` for `enum T { ... }`

Special case: `struct A;` is treated as `Opaque` and will not implement `Struct`.

### Implementation control

You can disable specific impls via attributes; then you must provide them manually.

```rust, ignore
#[derive(Reflect)]
#[reflect(TypePath = false, Typed = false)]
struct Foo { /* ... */ }
```

All of the above toggles can be turned off; turning them on explicitly is meaningless because it is the default.

Because `TypePath` often needs customization, an attribute is provided to override the path:

```rust, ignore
#[derive(Reflect)]
#[reflect(type_path = "you::me::Foo")]
struct Foo { /* ... */ }
```

`Opaque` is a special attribute that forces the type to be treated as `Opaque` instead of `Struct`, etc.

```rust, ignore
#[derive(Reflect)]
#[reflect(Opaque)] // error
struct Foo { /* ... */ }
```

Unit structs like `struct A;` are also treated as `Opaque`. They have no internal data, so the macro can auto-generate `reflect_clone`, etc. But if you mark a type as `Opaque` yourself, the macro will not inspect its fields; therefore `Opaque` types are required to at least implement `Clone`.

```rust, ignore
#[derive(Reflect, Clone)]
#[reflect(Opaque, clone)] // error
struct Foo { /* ... */ }
```

### Using standard traits

If the type implements traits like `Hash` or `Clone`, the reflection impls can be simplified (often much faster). The macro cannot know this, so it does not assume them by default. Use attributes to declare availability so the macro can optimize. As noted, `Opaque` types must support `Clone`, so implement it and mark with `clone`.

```rust, ignore
#[derive(Reflect)]
#[reflect(Opaque, clone, hash)] // error
struct Foo { /* ... */ }
// impl Clone, Hash ...
```

Available flags:

- `clone`: std::Clone
- `default`: std::Default
- `hash`: std::Hash
- `partial_eq`: std::PartialEq
- `serialize`: serde::Serialize
- `deserialize`: serde::Deserialize

`auto_register` is special: with the feature enabled, marked types auto-register.

Two convenience bundles enable multiple flags at once:

- `serde`: `serialize` + `deserialize` + `auto_register`
- `full`: all seven above (including `auto_register`)

### Docs reflection

Enable the `reflect_docs` feature to include docs in type info. By default the macro collects `#[doc = "..."]` (including `///` comments).

Use `#[reflect(docs = false)]` to disable doc collection for a type.

Use `#[reflect(docs = "...")]` to override with custom docs; when present, the macro ignores `#[doc = "..."]`.

### `impl_reflect`

Implements reflection for foreign types, requiring full type info and field access. Due to the orphan rule, this is typically used inside the reflection crate itself.

```rust, ignore
impl_reflect! {
    #[reflect(full)]
    enum Option<T> {
        Some(T),
        None,
    }
}
```

### `impl_reflect_opaque`

Simplified macro for `Opaque` types. Syntax: `(in module_path as alias_name) ident (..attrs..)`.

Examples:

```rust, ignore
impl_reflect_opaque!(u64 (full));
impl_reflect_opaque!(String (clone, debug, TypePath = false, docs = "hello"));
impl_reflect_opaque!((in utils::time) Instant (clone));
impl_reflect_opaque!((in utils::time as Ins) Instant (clone));
```

This macro always implies `Opaque`, so `clone` is required.
