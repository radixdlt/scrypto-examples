use scrypto::prelude::*;

import! {
r#"
{
    "package_address": "01a99c5f6d0f4b92e81968405bde0e14709ab6630dc0e215a38eef",
    "blueprint_name": "FlatAdmin",
    "functions": [
      {
        "name": "instantiate_flat_admin",
        "inputs": [
          {
            "type": "String"
          }
        ],
        "output": {
          "type": "Tuple",
          "elements": [
            {
              "type": "Custom",
              "name": "ComponentAddress",
              "generics": []
            },
            {
              "type": "Custom",
              "name": "Bucket",
              "generics": []
            }
          ]
        }
      }
    ],
    "methods": [
      {
        "name": "create_additional_admin",
        "mutability": "Mutable",
        "inputs": [],
        "output": {
          "type": "Custom",
          "name": "Bucket",
          "generics": []
        }
      },
      {
        "name": "destroy_admin_badge",
        "mutability": "Mutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "Bucket",
            "generics": []
          }
        ],
        "output": {
          "type": "Unit"
        }
      },
      {
        "name": "get_admin_badge_address",
        "mutability": "Immutable",
        "inputs": [],
        "output": {
          "type": "Custom",
          "name": "ResourceAddress",
          "generics": []
        }
      }
    ]
  }
"#
}

blueprint! {
    struct ManagedAccess {
        admin_badge: ResourceAddress,
        flat_admin_controller: ComponentAddress,
        protected_vault: Vault,
    }

    impl ManagedAccess {
        pub fn instantiate_managed_access() -> (ComponentAddress, Bucket) {
            let (flat_admin_component, admin_badge) =
                FlatAdmin::instantiate_flat_admin("My Managed Access Badge".into());

            let auth = AccessRules::new()
                .method("withdraw_all", auth!(require(admin_badge.resource_address())))
                .default(auth!(allow_all));

            let component = Self {
                admin_badge: admin_badge.resource_address(),
                flat_admin_controller: flat_admin_component,
                protected_vault: Vault::new(RADIX_TOKEN),
            }
            .instantiate()
            .add_access_check(auth)
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
            self.flat_admin_controller
        }
    }
}
