use scrypto::prelude::*;

import! {
r#"
{
  "package_address": "package_sim1q9h0dr0z36zaq6h66lg5putxtztyf0sgelxu654r67ks765aue",
  "blueprint_name": "FlatAdmin",
  "abi": {
    "structure": {
      "type": "Struct",
      "name": "FlatAdmin",
      "fields": {
        "type": "Named",
        "named": [
          [
            "admin_mint_badge",
            {
              "type": "Custom",
              "type_id": 179,
              "generics": []
            }
          ],
          [
            "admin_badge",
            {
              "type": "Custom",
              "type_id": 182,
              "generics": []
            }
          ]
        ]
      }
    },
    "fns": [
      {
        "ident": "instantiate_flat_admin",
        "mutability": null,
        "input": {
          "type": "Struct",
          "name": "FlatAdmin_instantiate_flat_admin_Input",
          "fields": {
            "type": "Named",
            "named": [
              [
                "arg0",
                {
                  "type": "String"
                }
              ]
            ]
          }
        },
        "output": {
          "type": "Tuple",
          "elements": [
            {
              "type": "Custom",
              "type_id": 129,
              "generics": []
            },
            {
              "type": "Custom",
              "type_id": 177,
              "generics": []
            }
          ]
        },
        "export_name": "FlatAdmin_instantiate_flat_admin"
      },
      {
        "ident": "create_additional_admin",
        "mutability": "Mutable",
        "input": {
          "type": "Struct",
          "name": "FlatAdmin_create_additional_admin_Input",
          "fields": {
            "type": "Named",
            "named": []
          }
        },
        "output": {
          "type": "Custom",
          "type_id": 177,
          "generics": []
        },
        "export_name": "FlatAdmin_create_additional_admin"
      },
      {
        "ident": "destroy_admin_badge",
        "mutability": "Mutable",
        "input": {
          "type": "Struct",
          "name": "FlatAdmin_destroy_admin_badge_Input",
          "fields": {
            "type": "Named",
            "named": [
              [
                "arg0",
                {
                  "type": "Custom",
                  "type_id": 177,
                  "generics": []
                }
              ]
            ]
          }
        },
        "output": {
          "type": "Unit"
        },
        "export_name": "FlatAdmin_destroy_admin_badge"
      },
      {
        "ident": "get_admin_badge_address",
        "mutability": "Immutable",
        "input": {
          "type": "Struct",
          "name": "FlatAdmin_get_admin_badge_address_Input",
          "fields": {
            "type": "Named",
            "named": []
          }
        },
        "output": {
          "type": "Custom",
          "type_id": 182,
          "generics": []
        },
        "export_name": "FlatAdmin_get_admin_badge_address"
      }
    ]
  }
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

            let rules = AccessRules::new()
                .method("withdraw_all", rule!(require(admin_badge.resource_address())))
                .default(rule!(allow_all));

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
