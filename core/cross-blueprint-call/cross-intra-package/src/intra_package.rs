use crate::airdrop::airdrop::Airdrop;
use scrypto::prelude::*;

#[blueprint]
mod intra_package {
    struct IntraPackageCallGlobal {
        airdrop: Global<Airdrop>,
    }

    impl IntraPackageCallGlobal {
        pub fn instantiate_proxy() -> Global<IntraPackageCallGlobal> {
            let (address_reservation, _component_address) =
                Runtime::allocate_component_address(Runtime::blueprint_id());

            return Self {
                airdrop: Airdrop::instantiate_airdrop(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(address_reservation)
            .globalize();
        }

        pub fn free_token(&self) -> Bucket {
            // Calling a method on a component using `.method_name()`.
            self.airdrop.free_token()
        }

        pub fn instantiate_airdrop(&mut self) -> Global<Airdrop> {
            return Airdrop::instantiate_airdrop();
        }
    }
}
