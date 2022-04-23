# Managed Access
This example demonstrates the use of the `FlatAdmin` blueprint to manage access in another blueprint.

Note that in order for this example to function, you will have to publish the package containing the `FlatAdmin` blueprint to your simulator to a specific address (or change the address imported near the top of `lib.rs` in this package).

If you wish to publish `FlatAdmin` to the appropriate address, switch to that directory and run:
```bash
resim publish . --address 01a99c5f6d0f4b92e81968405bde0e14709ab6630dc0e215a38eef
```

## Importing & Calling a Blueprint
Currently, importing another blueprint requires a few manual steps.  We expect to simplify this process in the future, but for now here are the steps:

1. Publish the package containing the blueprint you wish to import.
2. Export the ABI for that blueprint using the command `resim export-abi <package_address> <blueprint_name>`
3. Copy the output of that command, and paste it into the source file you wish to consume it from.  Enclose the content within an opening `import! {
r#"` and enclosing `"#}` block.  Example:
```rust
use scrypto::prelude::*;

import! {
r#"
<EXPORTED_ABI>
"#
}
```

Now you'll be able to call functions on that blueprint like so: `FlatAdmin::some_function(<args>)`

## Resources and Data
```rust
struct ManagedAccess {
  admin_badge: ResourceAddress,
  flat_admin_controller: ComponentAddress,
  protected_vault: Vault,
}
```

Our instantiated component will maintain a single vault which stores XRD.  Anyone may deposit to the vault, but only a caller in possession of an admin badge may withdraw from it.

The only state we need to maintain is the aforementioned vault, and the `ResourceAddress` of the badge used for authorization.  As a convenience for the user, we will also store the address of the `FlatAdmin` component which manages the supply of those badges.

## Getting Ready for Instantiation
In order to instantiate, we'll require no parameters and return to the caller a tuple containing the address of the newly instantiated component, and a bucket containing the first admin badge created by our `FlatAdmin` badge manager:
```rust
pub fn instantiate_managed_access() -> (ComponentAddress, Bucket) {
```

Our first step will be to instantiate a `FlatAdmin` component, and store the results of that instantiation.

```rust
let (flat_admin_component, admin_badge) =
  FlatAdmin::instantiate_flat_admin("My Managed Access Badge".into());
```

We then need to specify that only a holder of the admin badge may withdraw funds from a managed access component. 

```rust
let auth = AccessRules::new()
  .method("withdraw_all", auth!(require(admin_badge.resource_address())))
  .default(auth!(allow_all));
```

That gives us everything we need to populate our `struct`, instantiate, and return the results to our caller:

```rust
let component = Self {
    admin_badge: admin_badge.resource_address(),
    flat_admin_controller: flat_admin_component,
    protected_vault: Vault::new(RADIX_TOKEN),
}
.instantiate()
.add_access_check(auth)
.globalize();
(component, admin_badge)
```        

## Adding Methods
First, we'll create a protected method to allow withdrawal. This method is protected through the v0.4.0 system-level authentication system. Meaning, the system will automatically check that an admin badge is present in the Auth Zone before allowing the call to take place. 

```rust
pub fn withdraw_all(&mut self) -> Bucket {
  self.protected_vault.take_all()
}
```

The rest of the methods are straightforward.  We'll add a method to permit anyone to deposit XRD, and then some read-only methods to return data about our admin badge and the `FlatAdmin` controller which manages the supply of badges.

```rust
pub fn deposit(&mut self, to_deposit: Bucket) {
  self.protected_vault.put(to_deposit);
}

pub fn get_admin_badge_address(&self) -> Address {
  self.admin_badge.address()
}

pub fn get_flat_admin_controller_address(&self) -> Address {
  self.flat_admin_controller
}
```

That's it.  Access control components like `FlatAdmin` are expected to be very commonly consumed by other blueprints, as they provide consistent, re-usable mechanisms to manage privileges.