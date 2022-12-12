# Cross-Blueprint Call

When the complexity of a DeFi application increases, it's sometimes impossible to put all logic into one blueprint. Instead, we need a group of modular blueprints, each including a distinct subset of the business logic.

In this example, we demonstrate two ways of calling a function or method defined in a different blueprint, depending on where it's located.

## Callee Is From A Different Package

If the function or method is from an already published package, we need to import the external package and component definition. e.g,
    
```rust
external_blueprint! {
    AirdropPackageTarget {
        fn instantiate_airdrop() -> ComponentAddress;
    }
}

external_component! {
    AirdropComponentTarget {
        fn free_token(&mut self) -> Bucket;
    }
}
```

Once the package and component definition has been imported, we can then call functions on a blueprint of that package, for example,

```rust
let airdrop_component = AirdropPackageTarget::at(airdrop_package_address, "Airdrop").instantiate_airdrop();
```

To call a method, though, we need a component address, which can be parsed from string or taken from the method parameters.
```rust
let address = ComponentAddress::from_str("01e9917332573b6332ffaaadc96bc1509cc24ef8aa69d1cd117d39").unwrap();
let airdrop: AirdropComponentTarget = AirdropComponentTarget::at(address);
let received_tokens = airdrop.free_token();
```

## Callee Is From This Package

If the function or method you're calling is from the same package, we can import the blueprint using Rust's `use` keyword.

In our example package, we have the following files:
```
├─ src
│   ├─ lib.rs
│   ├─ airdrop.rs
│   ├─ cross_package.rs
│   └─ intra_package.rs
├─ test
│   └─ lib.rs
└─ Cargo.toml
```

In `intra_package.rs`, we write

```rust
use crate::airdrop::*;
```

This gives us access to the `AirdropComponent` type which allows us to call the functions and methods defined on the blueprint.

Notice that in this example, instead of storing the ComponentAddress in the state, we can directly store a `AirdropComponent`. This is because the airdrop component has not been globalized (we called `instantiate_airdrop_local` instead of `instantiate_airdrop`).