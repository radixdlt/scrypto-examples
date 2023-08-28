## Callee Is From This Package
If you have a different blueprint but from within the same package that you'd like to incorporate, you must make sure that we can import the blueprint using Rust's `use` keyword.

In our example package, we have the following files:
```
├─ src
│   ├─ lib.rs
│   ├─ airdrop.rs
│   ├─ intra-package-local.rs
│   └─ intra_package.rs
├─ test
│   └─ lib.rs
└─ Cargo.toml
```

In `intra_package.rs` or `intra-package-local`, we write

```rust
use crate::airdrop::airdrop::Airdrop;
```

This gives us access to the `Global<Airdrop>` type which allows us to call the functions and methods defined on the blueprint.

Notice that in this example, instead of storing the `ComponentAddress` in the state, we can directly store a `Global<Airdrop>`. This is because the airdrop component has not been globalized.