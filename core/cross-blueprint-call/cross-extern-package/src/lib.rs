use scrypto::prelude::*;


#[blueprint]
mod extern_blueprint_call {
    extern_blueprint!(
        "package_sim1p4kwg8fa7ldhwh8exe5w4acjhp9v982svmxp3yqa8ncruad4rv980g",
        Airdrop {
            fn instantiate_airdrop() -> Global<Airdrop>;
            fn instantiate_airdrop_local() -> Owned<Airdrop>;
            fn free_token(&mut self) -> Bucket;
        }
    );

    struct ExternBlueprintCall {}

    impl ExternBlueprintCall {
        pub fn instantiate_proxy() -> Global<ExternBlueprintCall> {
            Self {
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn free_token(&self) -> Bucket {
            // Retrieving Airdrop component
            let mut airdrop_component: Global<Airdrop> = global_component!(
                Airdrop,
                "component_sim1cpz9230mp9jp2e5fqxg6p2700txlude7mk5genwyk6ktaxxgfwmz7c"
            );
            // Calling a method on a component using `.free_token()`.
            return airdrop_component.free_token()
        }
    }
}
