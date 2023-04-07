use crate::airdrop::*;
use scrypto::prelude::*;

#[blueprint]
mod proxy2 {
    struct Proxy2 {
        airdrop: airdrop::AirdropComponent,
    }

    impl Proxy2 {
        pub fn instantiate_proxy() -> ComponentAddress {
            Self {
                airdrop: airdrop::AirdropComponent::instantiate_airdrop_local(),
            }
            .instantiate()
            .globalize()
        }

        pub fn free_token(&self) -> Bucket {
            // Calling a method on a component using `.method_name()`.
            self.airdrop.free_token()
        }
    }
}
