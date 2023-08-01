# Cross-Blueprint Call
In this example, we demonstrate how to call a blueprint and component from a different package.

## Callee Is From A Different Package

If the function or method is from an already deployed package, we need to import the external package and component definition. e.g,
    
```rust
extern_blueprint!(
    "package_sim1p40mzz4yg6n4gefzq5teg2gsts63wmez00826p8m5eslr864fr3648", <1>
    Airdrop {
        fn instantiate_airdrop() -> Global<Airdrop>;
        fn instantiate_airdrop_local() -> Owned<Airdrop>;
        fn free_token(&mut self) -> Bucket;
    }
);
```
<1> For this example, since we are using the simulator, make sure the Airdrop blueprint has been published locally from the `cross-intra-package` folder. Also ensure that the `PackageAddress` is correctly hardcoded.

Once the package and component definition has been imported, we can then call functions on a blueprint of that package, for example,

```rust
Blueprint::<Airdrop>::instantiate_airdrop()
```

We will instantiate the component and save it in our blueprint struct like so:

```rust
struct ExternBlueprintCall {
    airdrop: Global<Airdrop>,
}

impl ExternBlueprintCall {
    pub fn instantiate_proxy() -> Global<ExternBlueprintCall> {
        Self {
            airdrop: Blueprint::<Airdrop>::instantiate_airdrop()
        }
        .instantiate()
        .prepare_to_globalize(OwnerRole::None)
        .globalize()
    }
}
```

We can now call methods from the `Global<Airdrop>` component:

```rust
pub fn free_token(&mut self) -> Bucket {
    // Retrieving Airdrop component
    // Calling a method on a component using `.free_token()`.
    self.airdrop.free_token()
}
```

