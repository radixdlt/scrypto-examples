use scrypto::prelude::*;

use crate::airdrop::Airdrop;

blueprint! {
    struct Proxy2 {
        airdrop: Airdrop,
    }

    impl Proxy2 {
        pub fn instantiate_proxy() -> ComponentAddress {
            Self {
                // The instantiate_airdrop() function returns a generic Component. We use `.into()` to convert it into an `Airdrop`.
                airdrop: Airdrop::instantiate_airdrop().into(),
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
