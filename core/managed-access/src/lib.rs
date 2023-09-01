use scrypto::prelude::*;

#[blueprint]
mod managed_access {
    extern_blueprint!(
        "package_sim1p4kwg8fa7ldhwh8exe5w4acjhp9v982svmxp3yqa8ncruad4rv980g",
        FlatAdmin {
            fn instantiate_flat_admin(badge_name: String) -> (Global<FlatAdmin>, Bucket);
            fn create_additional_admin(&mut self) -> Bucket;
            fn destroy_admin_badge(&mut self, to_destroy: Bucket);
            fn get_admin_badge_address(&self) -> ResourceAddress;
        }
    );

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
    struct ManagedAccess {
        admin_badge: ResourceAddress,
        flat_admin_controller: Global<FlatAdmin>,
        protected_vault: Vault,
    }

    impl ManagedAccess {
        pub fn instantiate_managed_access(badge_name: String) -> (Global<ManagedAccess>, Bucket) {
            let (flat_admin_component, admin_badge): (Global<FlatAdmin>, Bucket) =
                Blueprint::<FlatAdmin>::instantiate_flat_admin(badge_name);

            let component = Self {
                admin_badge: admin_badge.resource_address(),
                flat_admin_controller: flat_admin_component,
                protected_vault: Vault::new(XRD),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles!(
                admin => rule!(require(admin_badge.resource_address()));
            ))
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

        pub fn get_flat_admin_controller_address(&self) -> Global<FlatAdmin> {
            self.flat_admin_controller
        }
    }
}
