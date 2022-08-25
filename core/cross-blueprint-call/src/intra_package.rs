use crate::airdrop::*;
use scrypto::prelude::*;

blueprint! {
    struct Proxy2 {
        airdrop: AirdropComponent,
    }

    impl Proxy2 {
        pub fn instantiate_proxy() -> ComponentAddress {
            Self {
                // The instantiate_airdrop() function returns a generic Component. We use `.into()` to convert it into an `Airdrop`.
                airdrop: AirdropComponent {
                    component: scrypto::component::Component::try_from(
                        AirdropComponent::instantiate_airdrop().to_vec().as_slice(),
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
