use scrypto::prelude::*;

// This is a simple Airdrop blueprint. All components instantiated from it will initially
// hold 1000 FreeToken within a vault. When the `free_token` method is called, 1 FreeToken will be
// taken from the vault and returned to the caller.

#[blueprint]
mod airdrop {
    struct Airdrop {
        tokens: Vault,
    }

    impl Airdrop {
        pub fn instantiate_airdrop() -> ComponentAddress {
            // .globalize makes the component accessible globally through a public component address
            Self::instantiate_airdrop_local()
                .globalize()
        }

        pub fn instantiate_airdrop_local() -> AirdropComponent {
            // Simply instantiating the component (without globalizing it) makes its methods
            // not callable from outside. In this case, it has to be owned by a particular component. Only that
            // component will be able to call methods on it. You can see an example of this in `intra_package.rs`

            Self {
                tokens: Vault::with_bucket(
                    ResourceBuilder::new_fungible()
                        .divisibility(DIVISIBILITY_MAXIMUM)
                        .metadata("name", "FreeToken")
                        .initial_supply(1000),
                ),
            }
            .instantiate()
        }

        pub fn free_token(&mut self) -> Bucket {
            // Take 1 FreeToken and return
            self.tokens.take(1)
        }
    }
}
