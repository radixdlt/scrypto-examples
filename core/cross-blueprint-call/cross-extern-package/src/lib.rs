use scrypto::prelude::*;


#[blueprint]
mod extern_blueprint_call {
    extern_blueprint!(
        "package_sim1p40mzz4yg6n4gefzq5teg2gsts63wmez00826p8m5eslr864fr3648",
        Airdrop {
            fn instantiate_airdrop() -> Global<Airdrop>;
            fn instantiate_airdrop_local() -> Owned<Airdrop>;
            fn free_token(&mut self) -> Bucket;
        }
    );

    struct ExternBlueprintCall {
        airdrop: Global<Airdrop>,
    }

    impl ExternBlueprintCall {
        pub fn instantiate_proxy() -> Global<ExternBlueprintCall> {
            Self {
                airdrop: Blueprint::<Airdrop>::instantiate_airdrop()
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn free_token(&mut self) -> Bucket {
            // Retrieving Airdrop component
            // Calling a method on a component using `.free_token()`.
            self.airdrop.free_token()
        }
    }
}
