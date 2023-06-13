use scrypto::prelude::*;


#[blueprint]
mod proxy1 {
    extern_blueprint!(
        "package_sim1p4kwg8fa7ldhwh8exe5w4acjhp9v982svmxp3yqa8ncruad4rv980g",
        AirdropBlueprintTarget {
            fn instantiate_airdrop() -> Global<Airdrop>;
            fn instantiate_airdrop_local() -> Owned<Airdrop>;
            fn free_token(&mut self) -> Bucket;
        }
    );

    const AIRDROP: Global<Airdrop> = global_component!(
        AirdropBlueprintTarget,
        "component_sim1cpz9230mp9jp2e5fqxg6p2700txlude7mk5genwyk6ktaxxgfwmz7c"
    );

    struct Proxy1 {}

    impl Proxy1 {
        pub fn instantiate_proxy() -> Global<Proxy1> {
            Self {
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn free_token(&self) -> Bucket {
            // Calling a method on a component using `.free_token()`.
            let mut airdrop = AIRDROP.free_token();
            return airdrop
        }
    }
}
