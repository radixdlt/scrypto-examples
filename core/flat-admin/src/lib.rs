use scrypto::prelude::*;

#[blueprint]
mod flat_admin {
    enable_method_auth! {
        roles {
            admin
        },
        methods {
            create_additional_admin => admin;
            destroy_admin_badge => PUBLIC;
            get_admin_badge_address => PUBLIC;
        }
    }
    struct FlatAdmin {
        first_admin_badge: ResourceManager,
    }

    impl FlatAdmin {
        pub fn instantiate_flat_admin(badge_name: String) -> (Global<FlatAdmin>, Bucket) {

            let blueprint_id = Runtime::blueprint_id();
            let (_global_address_reservation, component_address) = Runtime::allocate_component_address(blueprint_id);

            // Create the ResourceManager for a mutable supply admin badge
            let first_admin_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", badge_name)
                .mintable(rule!(require(global_caller(component_address))), LOCKED)
                .burnable(rule!(require(global_caller(component_address))), LOCKED)
                .mint_initial_supply(1);

            // Initialize our component, placing the minting authority badge within its vault, where it will remain forever
            let component = Self {
                first_admin_badge: first_admin_badge.resource_manager(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updateable(rule!(require(first_admin_badge.resource_address()))))
            .roles(
                roles!(
                    admin => rule!(require(first_admin_badge.resource_address())), mutable_by: admin, admin;
                )
            )
            .globalize();

            // Return the instantiated component and the admin badge we just minted
            (component, first_admin_badge)
        }

        // Any existing admin may create another admin token
        pub fn create_additional_admin(&mut self) -> Bucket {
            // The "authorize" method provides a convenient shortcut to make use of the mint authority badge within our vault without removing it
            return self.first_admin_badge.mint(dec!(1));
        }

        pub fn destroy_admin_badge(&mut self, to_destroy: Bucket) {
            assert!(
                to_destroy.resource_address() == self.first_admin_badge.resource_address(),
                "Can not destroy the contents of this bucket!"
            );
            to_destroy.burn();
        }

        pub fn get_admin_badge_address(&self) -> ResourceAddress {
            return self.first_admin_badge.resource_address();
        }
    }
}
