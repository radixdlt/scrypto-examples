use scrypto::prelude::*;

#[blueprint]
mod radiswap {
    struct Radiswap {
        liquidity_pool_component: Global<TwoResourcePool>,
    }

    impl Radiswap {
        pub fn instantiate_pool(
            owner_role: OwnerRole,
            token_a: ResourceAddress,
            token_b: ResourceAddress,
        ) -> Global<Radiswap> {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Radiswap::blueprint_id());

            let global_component_caller_badge =
                NonFungibleGlobalId::global_caller_badge(component_address);

            // Creating a new pool will check the following for us:
            // 1. That both resources are not the same.
            // 2. That none of the resources are non-fungible
            let liquidity_pool_component = Blueprint::<TwoResourcePool>::instantiate(
                owner_role.clone(),
                rule!(require(global_component_caller_badge)),
                (token_a, token_b),
                None,
            );

            // Instantiate our Radiswap component
            Self {
                liquidity_pool_component,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(address_reservation)
            .globalize()
        }

        /// Adds liquidity to this pool and return the LP tokens representing pool shares
        /// along with any remainder.
        pub fn add_liquidity(
            &mut self,
            token_a: Bucket,
            token_b: Bucket,
        ) -> (Bucket, Option<Bucket>) {
            // All the checks for correctness of buckets and everything else is handled by the pool
            // component! Just pass it the resources and it will either return the pool units back
            // if it succeeds or abort on failure.
            self.liquidity_pool_component.contribute((token_a, token_b))
        }

        /// This method does not need to be here - the pool units are redeemable without it by the
        /// holders of the pool units directly from the pool. In this case this is just a nice proxy
        /// so that users are only interacting with one component and do not need to know about the
        /// address of Radiswap and the address of the Radiswap pool.
        pub fn remove_liquidity(&mut self, pool_units: Bucket) -> (Bucket, Bucket) {
            self.liquidity_pool_component.redeem(pool_units)
        }

        /// Swaps token A for B, or vice versa.
        pub fn swap(&mut self, input_bucket: Bucket) -> Bucket {
            let mut reserves = self.vault_reserves();

            let input_amount = input_bucket.amount();

            let input_reserves = reserves
                .remove(&input_bucket.resource_address())
                .expect("Resource does not belong to the pool");

            let (output_resource_address, output_reserves) = reserves.into_iter().next().unwrap();

            let output_amount = (input_amount.checked_mul(output_reserves))
                .and_then(|d| d.checked_div(input_reserves.checked_add(input_amount).unwrap()));

            // NOTE: It's the responsibility of the user of the pool to do the appropriate rounding
            // before calling the withdraw method.

            self.deposit(input_bucket);
            self.withdraw(output_resource_address, output_amount.unwrap())
        }

        pub fn vault_reserves(&self) -> IndexMap<ResourceAddress, Decimal> {
            self.liquidity_pool_component.get_vault_amounts()
        }

        pub fn deposit(&mut self, bucket: Bucket) {
            self.liquidity_pool_component.protected_deposit(bucket)
        }

        pub fn withdraw(&mut self, token_address: ResourceAddress, amount: Decimal) -> Bucket {
            self.liquidity_pool_component.protected_withdraw(
                token_address,
                amount,
                WithdrawStrategy::Rounded(RoundingMode::ToZero),
            )
        }
    }
}
