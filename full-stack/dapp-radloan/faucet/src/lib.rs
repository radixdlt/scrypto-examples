use scrypto::prelude::*;

#[blueprint]
mod faucet {
    struct Faucet {}

    impl Faucet {
        pub fn instantiate() -> ComponentAddress {
            Self {}.instantiate().globalize()
        }
    }
}
