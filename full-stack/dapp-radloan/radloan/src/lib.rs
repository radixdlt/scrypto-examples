use scrypto::prelude::*;

#[blueprint]
mod radloan {
    struct RadLoan {}

    impl RadLoan {
        pub fn instantiate() -> ComponentAddress {
            Self {}.instantiate().globalize()
        }
    }
}
