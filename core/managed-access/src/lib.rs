use scrypto::prelude::*;

#[blueprint]
mod managed_access {
    const TARGET_PACKAGE_ADDRESS: PackageAddress = PackageAddress::new_or_panic([
        13, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1,
    ]);

    extern_blueprint!(
        TARGET_PACKAGE_ADDRESS,
        FlatAdmin {
            fn instantiate_flat_admin(badge_name: String);
            fn create_additional_admin(&mut self);
            fn destroy_admin_badge(&mut self, to_destroy: Bucket);
            fn get_admin_badge_address(&self);
        }
    );
    enable_method_auth! {
        roles {
            admin
        },
        methods {
            withdraw_all => admin;
            deposit => PUBLIC;
            get_admin_badge_address => PUBLIC;
            get_flat_admin_controller_address => PUBLIC;
        }
    }
    struct ManagedAccess {
        admin_badge: ResourceAddress,
        flat_admin_controller: Global<FlatAdmin>,
        protected_vault: Vault,
    }

    impl ManagedAccess {
        pub fn instantiate_managed_access(
            flat_admin_package_address: PackageAddress,
        ) -> (Global<ManagedAccess>, Bucket) {
            let (flat_admin_component, admin_badge): (Global<FlatAdmin>, Bucket) = Runtime::call_function(
                flat_admin_package_address, 
                "FlatAdmin", 
                "instantiate_flat_admin", 
                scrypto_args!("Admin Badge")
            );
            
            // let (flat_admin_component, admin_badge) =
                // FlatAdminPackageTarget::at(flat_admin_package_address, "FlatAdmin")
                //     .instantiate_flat_admin("My Managed Access Badge".into());


            let component = Self {
                admin_badge: admin_badge.resource_address(),
                flat_admin_controller: flat_admin_component,
                protected_vault: Vault::new(RADIX_TOKEN),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(
                roles!(
                    admin => rule!(require(admin_badge.resource_address())), mutable_by: admin, admin;
                )
            )
            .globalize();

            (component, admin_badge)
        }

        pub fn withdraw_all(&mut self) -> Bucket {
            self.protected_vault.take_all()
        }

        pub fn deposit(&mut self, to_deposit: Bucket) {
            self.protected_vault.put(to_deposit);
        }

        pub fn get_admin_badge_address(&self) -> ResourceAddress {
            self.admin_badge
        }

        pub fn get_flat_admin_controller_address(&self) -> ComponentAddress {
            self.flat_admin_controller.component_address()
        }
    }
}
