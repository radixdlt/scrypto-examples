use scrypto::prelude::*;

#[blueprint]
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
    struct FlatAdmin {
        admin_badge: ResourceManager,
    }

    impl FlatAdmin {
        pub fn instantiate_flat_admin(badge_name: String) -> (Global<FlatAdmin>, FungibleBucket) {
            // Creating a GlobalAddressReservation and ComponentAddress to use for the actor virtual badge pattern.
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(FlatAdmin::blueprint_id());

            // Create the ResourceManager for a mutable supply admin badge
            let admin_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata!(
                    init {
                        "name" => badge_name, locked;
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(deny_all);
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(require(global_caller(component_address)));
                    burner_updater => rule!(deny_all);
                ))
                .mint_initial_supply(1);

            // Initialize our component, placing the minting authority badge within its vault, where it will remain forever
            let component = Self {
                admin_badge: admin_badge.resource_manager(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(
                admin_badge.resource_address()
            ))))
            .roles(roles!(
                admin => rule!(require(admin_badge.resource_address()));
            ))
            .with_address(address_reservation)
            .globalize();

            // Return the instantiated component and the admin badge we just minted
            (component, admin_badge)
        }

        // Any existing admin may create another admin token
        pub fn create_additional_admin(&mut self) -> Bucket {
            // The "authorize" method provides a convenient shortcut to make use of the mint authority badge within our vault without removing it
            return self.admin_badge.mint(dec!(1));
        }

        pub fn destroy_admin_badge(&mut self, to_destroy: Bucket) {
            assert!(
                to_destroy.resource_address() == self.admin_badge.address(),
                "Can not destroy the contents of this bucket!"
            );
            to_destroy.burn();
        }

        pub fn get_admin_badge_address(&self) -> ResourceAddress {
            return self.admin_badge.address();
        }
    }
}
