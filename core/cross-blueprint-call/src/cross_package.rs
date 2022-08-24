use scrypto::prelude::*;

external_blueprint! {
    {
        package: "package_sim1qx7304nj09gmp8t8kd2xv6t9yxg7vug8su6wq9kjfutqzl29ru",
        blueprint: "Airdrop"
    },
    Airdrop {
        fn instantiate_airdrop() -> ComponentAddress;
        fn free_token(&mut self) -> Bucket;
    }
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
            let mut airdrop: Airdrop = self.airdrop.into();
            airdrop.free_token()
        }
    }
}
