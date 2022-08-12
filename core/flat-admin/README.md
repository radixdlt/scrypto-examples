# Flat Admin
This example demonstrates a blueprint which can be used by other blueprints and components to manage a set of admin badges for a single level of administration.

Instantiating a `FlatAdmin` component will create a badge manager component, as well as the first admin badge.  You can then use this badge for authorization into privileged methods on another component.  By interacting with the badge manager component, anyone possessing an admin badge can create an additional badge, which can be distributed as desired.

## Resources and Data
```rust
struct FlatAdmin {
    admin_mint_badge: Vault,
    admin_badge: ResourceAddress,
}
```

In order to be able to mint additional admin badges after our first, we'll need a vault to contain a badge which holds that minting permission.

For user convenience, we'll also maintain the `ResourceAddress` of the external admin badge that we'll be handing out, so that they can interrogate an instantiated `FlatAdmin` component about which badge it manages.

## Getting Ready for Instantiation
Upon instantiation, we'll only ask the user to name the badge.  We'll return to the user the instantiated component, as well as the first admin badge managed by the component.

```rust
pub fn instantiate_flat_admin(badge_name: String) -> (ComponentAddress, Bucket) {
```

We'll want our supply of admin badges to be mutable.  Mutable supply resources can only be minted and burned by an appropriate authority, so we'll first create a badge to serve as that authority, and then use that new badge to create our supply of admin badges.

```rust
let admin_mint_badge = ResourceBuilder::new_fungible()
    .divisibility(DIVISIBILITY_NONE)
    .initial_supply(1);

let mut admin_badge = ResourceBuilder::new_fungible()
    .divisibility(DIVISIBILITY_NONE)
    .metadata("name", badge_name)
    .mintable(rule!(require(admin_mint_badge.resource_address())), LOCKED)
    .burnable(rule!(require(admin_mint_badge.resource_address())), LOCKED)
    .no_initial_supply();
```

With that out of the way, we can mint our first admin badge, set the access rules for the component methods and create our component.  We'll tuck our sole minting authority badge safely away within its vault.  Then we'll return the new component and the admin badge.

```rust
let first_admin_badge = admin_mint_badge.authorize(|| {
    let admin_badge_manager = borrow_resource_manager!(admin_badge);
    admin_badge_manager.mint(1)
});

let rules: AccessRules = AccessRules::new()
    .method("create_additional_admin", rule!(require(admin_badge)))
    .default(rule!(allow_all));

let mut component = Self {
    admin_mint_badge: Vault::with_bucket(admin_mint_badge),
    admin_badge: admin_badge,
}
.instantiate();
component.add_access_check(rules);
let component_address = component.globalize();

(component_address, first_admin_badge)
```

## Allowing Users to Mint and Burn Admin Badges
In order for `FlatAdmin` to be more useful than just manually creating a single admin badge, it needs the capability to create and destroy admin badges.

Obviously we don't want just anyone to be able to create additional admin badges at will, so that privilege is protected by having to prove that you're already in possession of an admin badge. This protection is done through the `rules` defined above and added to the instantiated component.

```rust
pub fn create_additional_admin(&self) -> Bucket {
    self.admin_mint_badge.authorize(|| {
        let admin_badge_manager = borrow_resource_manager!(self.admin_badge);
        admin_badge_manager.mint(1)
    })
}
```

The `authorize` method is a convenience method which allows us to present the badge contained within our `admin_mint_badge` vault without having to fetch it, present it, and return it.  The closure syntax using `|` characters may be unfamiliar to you: think of `|auth|` as being equivalent to `(auth) ->` in Java or `(auth) =>` in C#.
