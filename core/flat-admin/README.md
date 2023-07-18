# Flat Admin
This example demonstrates a blueprint which can be used by other blueprints and components to manage a set of admin badges for a single level of administration.

Instantiating a `FlatAdmin` component will create a badge manager component, as well as the first admin badge.  You can then use this badge for authorization into privileged methods on another component.  By interacting with the badge manager component, anyone possessing an admin badge can create an additional badge, which can be distributed as desired. To do that we need to create a role based authorization within our component. We set that up by using the `enable_method_auth!` macro below our module.

## Enabling Method Auth
```rust
mod flat_admin {
    enable_method_auth! {
        roles {
            admin => updatable_by: [admin];
        },
        methods {
            create_additional_admin => restrict_to: [admin];
            destroy_admin_badge => PUBLIC;
            get_admin_badge_address => PUBLIC;
        }
    }
    ..
}
```

This macro defines our roles, and the method permissions. Most of our methods are set to `PUBLIC` meaning that it is accessible by anyone. However, `create_additional_admin` is mapped and restricted to the `admin` role we defined. This means only the `admin` can access this method. How this work will be configured later on.

Below will set up of the data our component will hold.

## Resources and Data
```rust
struct FlatAdmin {
    admin_badge: ResourceManager,
}
```

We store the `ResourceManager` of the admin badge so that we can continue to mint the badge to hand out.

## Getting Ready for Instantiation
Upon instantiation, we'll only ask the user to name the badge.  We'll return to the user the instantiated component, as well as the first admin badge managed by the component.

```rust
pub fn instantiate_flat_admin(badge_name: String) -> (Global<FlatAdmin>, Bucket) {
```

We'll also create a `GlobalAddressReservation` and `ComponentAddress` for our component to use the actor virtual badge pattern so that our component can have the authority to mint our admin badge, which we'll define later.

```rust
let (address_reservation, component_address) = 
    Runtime::allocate_component_address(Runtime::blueprint_id());
```

We'll want our supply of admin badges to be mutable.  Mutable supply resources can only be minted and burned by an appropriate authority, so we'll first create a badge to serve as that authority, and then use that new badge to create our supply of admin badges.

```rust
// Create the ResourceManager for a mutable supply admin badge.
let admin_badge = ResourceBuilder::new_fungible(OwnerRole::None)
    .divisibility(DIVISIBILITY_NONE)
    .metadata(metadata!(
            init {
                "name" => badge_name, locked;
            }
        )
    )
    .mint_roles(mint_roles!(
        minter => rule!(require(global_caller(component_address)));
        minter_updater => rule!(deny_all);
    ))
    .burn_roles(burn_roles!(
        burner => rule!(require(global_caller(component_address)));
        burner_updater => rule!(deny_all);
    ))
    .mint_initial_supply(1);
```

Now that we have our instantiation function mostly built out, it's time to populate our component struct and instantiate! Within the instantiation will be where we can configure the roles we defined earlier within the `enable_method_auth!` macro.

```rust
let component = Self {
    admin_badge: admin_badge.resource_manager(),
}
.instantiate()
.prepare_to_globalize(OwnerRole::Updatable(rule!(require(admin_badge.resource_address()))))
.roles(
    roles!(
        admin => rule!(require(first_admin_badge.resource_address()));
    )
)
.with_address(address_reservation)
.globalize();

(component, admin_badge)
```

There are a few things to note here:

1. We are using the first admin badge we mint as the owner of the component. This gives the first admin produced by the component to configure the metadata of the component.
2. We've configured the admin role we defined earlier to map to the admin badge. This means the admin must carry the admin badge to access the `create_additional_method` we restricted earlier.
3. We globalize the address with the `GlobalAddressReservation` we created earlier. This is so that the `ComponentAddress` that we receive from globalizing the component will be the same that will be generated from the the `GlobalAddressReservation`.


## Allowing Users to Mint and Burn Admin Badges
In order for `FlatAdmin` to be more useful than just manually creating a single admin badge, it needs the capability to create and destroy admin badges.

Obviously we don't want just anyone to be able to create additional admin badges at will, so that privilege is protected by having to prove that you're already in possession of an admin badge. This protection is done through the method auth we defined earlier and added to the instantiated component.

```rust
pub fn create_additional_admin(&mut self) -> Bucket {
    return self.admin_badge.mint(dec!(1));

}
```

Note: with resim, you can call a method protected by authorization rules by appending the `--proofs [quantity],[resource_address]` flag to the `resim call-method` command.

