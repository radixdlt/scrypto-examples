use scrypto::prelude::*;

#[blueprint]
mod flat_admin {
    struct FlatAdmin {
        admin_mint_badge: Vault,
        admin_badge: ResourceAddress,
    }

    impl FlatAdmin {
        pub fn instantiate_flat_admin(badge_name: String) -> (ComponentAddress, Bucket) {
            // Create a badge for internal use which will hold mint/burn authority for the admin badge we will soon create
            let admin_mint_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);

            // Create the ResourceManager for a mutable supply admin badge
            let first_admin_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", badge_name)
                .mintable(rule!(require(admin_mint_badge.resource_address())), LOCKED)
                .burnable(rule!(require(admin_mint_badge.resource_address())), LOCKED)
                .mint_initial_supply(1);

            // Setting uo the access rules of the component
            let rules: AccessRules = AccessRules::new()
                // The third parameter here specifies the authority allowed to update the rule.
                .method(
                    "create_additional_admin",
                    rule!(require(first_admin_badge.resource_address())),
                    LOCKED,
                )
                // The second parameter here specifies the authority allowed to update the rule.
                .default(AccessRule::AllowAll, AccessRule::DenyAll);

            // Initialize our component, placing the minting authority badge within its vault, where it will remain forever
            let mut component = Self {
                admin_mint_badge: Vault::with_bucket(admin_mint_badge),
                admin_badge: first_admin_badge.resource_address(),
            }
            .instantiate();
            component.add_access_check(rules);
            let component_address = component.globalize();

            // Return the instantiated component and the admin badge we just minted
            (component_address, first_admin_badge)
        }

        // Any existing admin may create another admin token
        pub fn create_additional_admin(&mut self) -> Bucket {
            // The "authorize" method provides a convenient shortcut to make use of the mint authority badge within our vault without removing it
            self.admin_mint_badge.authorize(|| {
                let admin_badge_manager = borrow_resource_manager!(self.admin_badge);
                admin_badge_manager.mint(1)
            })
        }

        pub fn destroy_admin_badge(&mut self, to_destroy: Bucket) {
            assert!(
                to_destroy.resource_address() == self.admin_badge,
                "Can not destroy the contents of this bucket!"
            );
            self.admin_mint_badge.authorize(|| {
                to_destroy.burn();
            })
        }

        pub fn get_admin_badge_address(&self) -> ResourceAddress {
            self.admin_badge
        }
    }
}
