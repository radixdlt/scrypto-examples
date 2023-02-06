use scrypto::prelude::*;

external_blueprint! {
    AirdropPackageTarget {
        fn instantiate_airdrop() -> ComponentAddress;
    }
}

external_component! {
    AirdropComponentTarget {
        fn free_token(&mut self) -> Bucket;
    }
}

#[blueprint]
mod proxy1 {
    struct Proxy1 {
        airdrop: ComponentAddress,
    }

    impl Proxy1 {
        pub fn instantiate_proxy(airdrop_package_address: PackageAddress) -> ComponentAddress {
            Self {
                // The instantiate_airdrop() function returns a generic ComponentAddress which we store to make calls
                // to the component at a later point.
                airdrop: AirdropPackageTarget::at(airdrop_package_address, "Airdrop").instantiate_airdrop(),
            }
            .instantiate()
            .globalize()
        }

        pub fn free_token(&self) -> Bucket {
            // Calling a method on a component using `.free_token()`.
            let mut airdrop: AirdropComponentTarget = AirdropComponentTarget::at(self.airdrop);
            airdrop.free_token()
        }
    }
}
