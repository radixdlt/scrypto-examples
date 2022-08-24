use crate::airdrop::*;
use scrypto::prelude::*;

blueprint! {
    struct Proxy2 {
        airdrop: Airdrop_Component,
    }

    impl Proxy2 {
        pub fn instantiate_proxy() -> ComponentAddress {
            Self {
                // The instantiate_airdrop() function returns a generic Component. We use `.into()` to convert it into an `Airdrop`.
                airdrop: Airdrop_Component {
                    component: scrypto::component::Component::try_from(
                        Airdrop_Component::instantiate_airdrop().to_vec().as_slice(),
                    )
                    .unwrap(),
                },
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
