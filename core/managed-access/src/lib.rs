use scrypto::prelude::*;

external_blueprint! {
  FlatAdminPackageTarget {
    fn instantiate_flat_admin(badge_name: String) -> (ComponentAddress, Bucket);
  }
}

external_component! {
    FlatAdminComponentTarget {
        fn create_additional_admin(&mut self) -> Bucket;
        fn destroy_admin_badge(&mut self, to_destroy: Bucket);
        fn get_admin_badge_address(&self) -> ResourceAddress;
    }
}

#[blueprint]
mod managed_access {
    struct ManagedAccess {
        admin_badge: ResourceAddress,
        flat_admin_controller: ComponentAddress,
        protected_vault: Vault,
    }

    impl ManagedAccess {
        pub fn instantiate_managed_access(
            flat_admin_package_address: PackageAddress,
        ) -> (ComponentAddress, Bucket) {
            let (flat_admin_component, admin_badge) =
                FlatAdminPackageTarget::at(flat_admin_package_address, "FlatAdmin")
                    .instantiate_flat_admin("My Managed Access Badge".into());

            let rules = AccessRules::new()
                .method(
                    "withdraw_all",
                    rule!(require(admin_badge.resource_address())),
                    AccessRule::DenyAll,
                )
                .default(rule!(allow_all), AccessRule::DenyAll);

            let mut component = Self {
                admin_badge: admin_badge.resource_address(),
                flat_admin_controller: flat_admin_component,
                protected_vault: Vault::new(RADIX_TOKEN),
            }
            .instantiate();
            component.add_access_check(rules);
            let component = component.globalize();

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
            self.flat_admin_controller
        }
    }
}
