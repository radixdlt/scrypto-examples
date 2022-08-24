use scrypto::prelude::*;

import! {
r#"
{
  "package_address": "package_sim1qx7304nj09gmp8t8kd2xv6t9yxg7vug8su6wq9kjfutqzl29ru",
  "blueprint_name": "Airdrop",
  "abi": {
    "structure": {
      "type": "Struct",
      "name": "Airdrop",
      "fields": {
        "type": "Named",
        "named": [
          [
            "tokens",
            {
              "type": "Custom",
              "type_id": 179,
              "generics": []
            }
          ]
        ]
      }
    },
    "fns": [
      {
        "ident": "instantiate_airdrop",
        "mutability": null,
        "input": {
          "type": "Struct",
          "name": "Airdrop_instantiate_airdrop_Input",
          "fields": {
            "type": "Named",
            "named": []
          }
        },
        "output": {
          "type": "Custom",
          "type_id": 129,
          "generics": []
        },
        "export_name": "Airdrop_instantiate_airdrop"
      },
      {
        "ident": "free_token",
        "mutability": "Mutable",
        "input": {
          "type": "Struct",
          "name": "Airdrop_free_token_Input",
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
        "export_name": "Airdrop_free_token"
      }
    ]
  }
}
"#
}

blueprint! {
    struct Proxy1 {
        airdrop: ComponentAddress,
    }

    impl Proxy1 {
        pub fn instantiate_proxy() -> ComponentAddress {
            Self {
                // The instantiate_airdrop() function returns a generic ComponentAddress which we store to make calls
                // to the component at a later point.
                airdrop: Airdrop::instantiate_airdrop(),
            }
            .instantiate()
            .globalize()
        }

        pub fn free_token(&self) -> Bucket {
            // Calling a method on a component using `.free_token()`.
            let airdrop: Airdrop = self.airdrop.into();
            airdrop.free_token()
        }
    }
}
