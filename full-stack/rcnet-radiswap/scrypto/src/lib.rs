use scrypto::prelude::*;

#[blueprint]
mod radiswap_module {
    use scrypto::blueprints::pool::{TWO_RESOURCE_POOL_INSTANTIATE_IDENT, TwoResourcePoolInstantiateInput, TwoResourcePoolContributeInput, TwoResourcePoolContributeOutput, TWO_RESOURCE_POOL_CONTRIBUTE_IDENT, TwoResourcePoolRedeemInput, TwoResourcePoolRedeemOutput, TWO_RESOURCE_POOL_REDEEM_IDENT, TwoResourcePoolGetVaultAmountsInput, TwoResourcePoolGetVaultAmountsOutput, TWO_RESOURCE_POOL_GET_VAULT_AMOUNTS_IDENT, TwoResourcePoolProtectedDepositInput, TwoResourcePoolProtectedDepositOutput, TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT, TwoResourcePoolProtectedWithdrawOutput, TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT, TwoResourcePoolProtectedWithdrawInput};

    struct Radiswap {
        fee: Decimal,
        pool_component: Global<AnyComponent>
    }

    impl Radiswap {
        /// Creates a new liquidity pool of the two tokens sent to the pool
        pub fn instantiate_radiswap(
            token_a: ResourceAddress,
            token_b: ResourceAddress,
            fee: Decimal,
        ) -> Global<Radiswap> {
            
            let (_, component_address) = 
                Runtime::allocate_component_address(Runtime::blueprint_id());

            let pool_component = Runtime::call_function(
                POOL_PACKAGE, 
                "TwoResourcePool", 
                TWO_RESOURCE_POOL_INSTANTIATE_IDENT, 
                scrypto_encode(&TwoResourcePoolInstantiateInput {
                    pool_manager_rule: rule!(require(global_caller(component_address))),
                    resource_addresses: (token_a, token_b),
                })
                .unwrap(),
            );

            // Create the Radiswap component and globalize it
            let radiswap = Self {
                fee: fee,
                pool_component: pool_component
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
            
            // Return the component address as well as the pool units tokens
            radiswap
        }

        pub fn swap(&mut self, input_bucket: Bucket) -> Bucket {
            let mut reserves = self.vault_reserves();

            let input_amount = input_bucket.amount();

            let input_reserves = reserves
                .remove(&input_bucket.resource_address())
                .expect("Resource does not belong to the pool");
            let (output_resource_address, output_reserves) = reserves.into_iter().next().unwrap();

            let output_amount = 
                (input_amount * output_reserves) 
                / (input_reserves + input_amount * (dec!("1") - self.fee));

            self.deposit(input_bucket);
            self.withdraw(output_resource_address, output_amount)
        }

        pub fn vault_reserves(&self) -> BTreeMap<ResourceAddress, Decimal> {
            self.pool_component
                .call::<TwoResourcePoolGetVaultAmountsInput, TwoResourcePoolGetVaultAmountsOutput>(
                    TWO_RESOURCE_POOL_GET_VAULT_AMOUNTS_IDENT,
                    &TwoResourcePoolGetVaultAmountsInput,
                )
        }

        fn deposit(&mut self, bucket: Bucket) {
            self.pool_component
                .call::<TwoResourcePoolProtectedDepositInput, TwoResourcePoolProtectedDepositOutput>(
                    TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
                    &TwoResourcePoolProtectedDepositInput { bucket }
                )
        }

        fn withdraw(&mut self, resource_address: ResourceAddress, amount: Decimal) -> Bucket {
            self.pool_component
                .call::<TwoResourcePoolProtectedWithdrawInput, TwoResourcePoolProtectedWithdrawOutput>(
                    TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                    &TwoResourcePoolProtectedWithdrawInput {
                        resource_address,
                        amount
                    }
                )
        }

        pub fn add_liquidity(
            &mut self,
            token_a: Bucket,
            token_b: Bucket,
        ) -> (Bucket, Option<Bucket>) {
            self.pool_component
                .call::<TwoResourcePoolContributeInput, TwoResourcePoolContributeOutput>(
                    TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    &TwoResourcePoolContributeInput {
                        buckets: (token_a, token_b),
                    },
                )
        }

        pub fn remove_liquidity(
            &mut self,
            pool_units: Bucket,
        ) -> (Bucket, Bucket) {
            self.pool_component
                .call::<TwoResourcePoolRedeemInput, TwoResourcePoolRedeemOutput>(
                    TWO_RESOURCE_POOL_REDEEM_IDENT,
                    &TwoResourcePoolRedeemInput { bucket: pool_units },
                )
        }

        pub fn get_swap_fee(&self) -> Decimal {
            return self.fee;
        }
    }
}