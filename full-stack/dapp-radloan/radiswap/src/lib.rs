use scrypto::prelude::*;

#[blueprint]
mod radiswap {
    struct RadiSwap {
        // Define what resources and data will be managed by RadiSwap components
    }

    impl RadiSwap {
        pub fn instantiate() -> ComponentAddress {
            Self {}.instantiate().globalize()
        }
    }
}
