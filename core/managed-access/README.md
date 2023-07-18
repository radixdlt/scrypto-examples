# Managed Access
This example demonstrates the use of the `FlatAdmin` blueprint to manage access in another blueprint.

Note that in order for this example to function, you will have to publish the package containing the `FlatAdmin` blueprint to your simulator to a specific address (or change the address imported near the top of `lib.rs` in this package).

Before trying this example, you should have created and account with `resim new-account` and created a simple badge used to manage the blueprint with `resim new-simple-badge`.

If you wish to publish `FlatAdmin`, switch to that directory and run:
```bash
resim publish . <owner_badge_NFAddress>
```

## Importing & Calling a Blueprint
Currently, importing another blueprint requires a few manual steps.  We expect to simplify this process in the future, but for now here are the steps:

1. Publish the package containing the blueprint you wish to import.
2. Define the functions of the blueprint with a call to the `extern_blueprint!` macro:

```rust
extern_blueprint!(
    "package_sim1p4kwg8fa7ldhwh8exe5w4acjhp9v982svmxp3yqa8ncruad4rv980g",
    FlatAdmin {
        fn instantiate_flat_admin(badge_name: String) -> (Global<FlatAdmin>, Bucket);
        fn create_additional_admin(&mut self) -> Bucket;
        fn destroy_admin_badge(&mut self, to_destroy: Bucket);
        fn get_admin_badge_address(&self) -> ResourceAddress;
    }
);
```

Now you'll be able to call functions on that blueprint like so: `Blueprint::<FlatAdmin>::instantiate_flat_admin(badge_name);`

## Enabling Method Authozation

We need to define our roles to specify who is allowed to withdraw funds from a managed access component. 

```rust
enable_method_auth! {
    roles {
        admin => updatable_by: [];
    },
    methods {
        withdraw_all => restrict_to: [admin];
        deposit => PUBLIC;
        get_admin_badge_address => PUBLIC;
        get_flat_admin_controller_address => PUBLIC;
    }
}
```

## Resources and Data
```rust
struct ManagedAccess {
    admin_badge: ResourceAddress,
    flat_admin_controller: Global<FlatAdmin>,
    protected_vault: Vault,
}
```

Our instantiated component will maintain a single vault which stores XRD.  Anyone may deposit to the vault, but only a caller in possession of an admin badge may withdraw from it.

The only state we need to maintain is the aforementioned vault, and the `ResourceAddress` of the badge used for authorization.  As a convenience for the user, we will also store the address of the `FlatAdmin` component which manages the supply of those badges.

## Getting Ready for Instantiation
In order to instantiate, we'll require the caller to specify the addess of the package containing the `FlagAdmin` blueprint and return to the caller a tuple containing the address of the newly instantiated component, and a bucket containing the first admin badge created by our `FlatAdmin` badge manager:
```rust
pub fn instantiate_managed_access(badge_name: String) -> (Global<ManagedAccess>, Bucket) {
```

Our first step will be to instantiate a `FlatAdmin` component, and store the results of that instantiation.

```rust
let 
(flat_admin_component, admin_badge): (Global<FlatAdmin>, Bucket) = 
Blueprint::<FlatAdmin>::instantiate_flat_admin(badge_name);
```


That gives us everything we need to populate our `struct`, instantiate, and return the results to our caller:

```rust
let component = Self {
    admin_badge: admin_badge.resource_address(),
    flat_admin_controller: flat_admin_component,
    protected_vault: Vault::new(RADIX_TOKEN),
}
.instantiate()
.prepare_to_globalize(OwnerRole::None)
.roles(
    roles!(
        admin => rule!(require(admin_badge.resource_address()));
    )
)
.globalize();

(component, admin_badge)
}
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

pub fn get_admin_badge_address(&self) -> ResourceAddress {
  self.admin_badge
}

pub fn get_flat_admin_controller_address(&self) -> ComponentAddress {
  self.flat_admin_controller
}
```

That's it.  Access control components like `FlatAdmin` are expected to be very commonly consumed by other blueprints, as they provide consistent, re-usable mechanisms to manage privileges.