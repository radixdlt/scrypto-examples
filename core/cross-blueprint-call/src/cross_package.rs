use scrypto::prelude::*;

import! {
r#"
{
    "package_address": "01e9917332573b6332ffaaadc96bc1509cc24ef8aa69d1cd117d39",
    "blueprint_name": "Airdrop",
    "functions": [
      {
        "name": "instantiate_airdrop",
        "inputs": [],
        "output": {
          "type": "Custom",
          "name": "ComponentAddress",
          "generics": []
        }
      }
    ],
    "methods": [
      {
        "name": "free_token",
        "mutability": "Mutable",
        "inputs": [],
        "output": {
          "type": "Custom",
          "name": "Bucket",
          "generics": []
        }
      }
    ]
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
                airdrop: Airdrop::instantiate_airdrop().into(),
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
