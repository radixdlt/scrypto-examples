use scrypto::prelude::*;

blueprint! {
    struct FlatAdmin {
        admin_mint_badge: Vault,
        admin_badge: ResourceAddress,
    }

    impl FlatAdmin {
        pub fn instantiate_flat_admin(badge_name: String) -> (ComponentAddress, Bucket) {
            // Create a badge for internal use which will hold mint/burn authority for the admin badge we will soon create
            let admin_mint_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);

            // Create the ResourceManager for a mutable supply admin badge
            let mut admin_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", badge_name)
                .mintable(auth!(require(admin_mint_badge.resource_address())), LOCKED)
                .burnable(auth!(require(admin_mint_badge.resource_address())), LOCKED)
                .no_initial_supply();

            // Using our minting authority badge, mint a single admin badge
            let first_admin_badge = admin_mint_badge.authorize(|| {
                let admin_badge_manager = borrow_resource_manager!(admin_badge);
                admin_badge_manager.mint(1)
            });

            // Setting uo the access rules of the component
            let auth: AccessRules = AccessRules::new()
                .method("create_additional_admin", auth!(require(admin_badge)))
                .default(auth!(allow_all));

            // Initialize our component, placing the minting authority badge within its vault, where it will remain forever
            let component = Self {
                admin_mint_badge: Vault::with_bucket(admin_mint_badge),
                admin_badge: admin_badge,
            }
            .instantiate()
            .add_access_check(auth)
            .globalize();

            // Return the instantiated component and the admin badge we just minted
            (component, first_admin_badge)
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
                let admin_badge_manager = borrow_resource_manager!(self.admin_badge);
                admin_badge_manager.burn(to_destroy)
            })
        }

        pub fn get_admin_badge_address(&self) -> ResourceAddress {
            self.admin_badge
        }
    }
}
