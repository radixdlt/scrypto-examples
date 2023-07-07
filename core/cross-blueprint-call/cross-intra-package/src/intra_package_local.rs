use crate::airdrop::airdrop::Airdrop;
use scrypto::prelude::*;

#[blueprint]
mod intra_package {
    struct IntraPackageCallOwned {
        airdrop: Owned<Airdrop>,
    }

    impl IntraPackageCallOwned {
        pub fn instantiate_proxy() -> Global<IntraPackageCallOwned> {
            return Self {
                airdrop: Airdrop::instantiate_airdrop_local(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }

        pub fn free_token(&self) -> Bucket {
            // Calling a method on a component using `.method_name()`.
            self.airdrop.free_token()
        }
    }
}
