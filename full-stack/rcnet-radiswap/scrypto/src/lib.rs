use scrypto::prelude::*;

#[blueprint]
mod radiswap_module {
    struct Radiswap {
        /// A vault containing pool reverses of reserves of token A.
        vault_a: Vault,
        /// A vault containing pool reverses of reserves of token B.
        vault_b: Vault,
        /// The token address of a token representing pool units in this pool
        pool_units_resource_manager: ResourceManager,
        /// The amount of fees imposed by the pool on swaps where 0 <= fee <= 1.
        fee: Decimal,
    }

    impl Radiswap {
        /// Creates a new liquidity pool of the two tokens sent to the pool
        pub fn instantiate_radiswap(
            bucket_a: Bucket,
            bucket_b: Bucket,
            fee: Decimal,
        ) -> (Global<Radiswap>, Bucket) {
            // Ensure that none of the buckets are empty and that an appropriate 
            // fee is set.
            assert!(
                !bucket_a.is_empty() && !bucket_b.is_empty(),
                "You must pass in an initial supply of each token"
            );

            assert!(
                fee >= dec!("0") && fee <= dec!("1"),
                "Invalid fee in thousandths"
            );

            let (_, component_address) = 
                Runtime::allocate_component_address(Runtime::blueprint_id());

            // Create the pool units token along with the initial supply specified  
            // by the user.
            let pool_units: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "Pool Unit")
                .metadata("symbol", "UNIT")
                .mintable(
                    rule!(require(global_caller(component_address))),
                    LOCKED,
                )
                .burnable(
                    rule!(require(global_caller(component_address))),
                    LOCKED,
                )
                .mint_initial_supply(100);

            // Create the Radiswap component and globalize it
            let radiswap = Self {
                vault_a: Vault::with_bucket(bucket_a),
                vault_b: Vault::with_bucket(bucket_b),
                pool_units_resource_manager: pool_units.resource_manager(),
                fee: fee,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
            
            // Return the component address as well as the pool units tokens
            (radiswap, pool_units)
        }

        /// Swaps token A for B, or vice versa.
        pub fn swap(&mut self, input_tokens: Bucket) -> Bucket {
            // Getting the vault corresponding to the input tokens and the vault 
            // corresponding to the output tokens based on what the input is.

            
            let (input_tokens_vault, output_tokens_vault): (&mut Vault, &mut Vault) =
                if input_tokens.resource_address() == 
                self.vault_a.resource_address() {
                    (&mut self.vault_a, &mut self.vault_b)
                } else if input_tokens.resource_address() == 
                self.vault_b.resource_address() {
                    (&mut self.vault_b, &mut self.vault_a)
                } else {
                    panic!(
                    "The given input tokens do not belong to this liquidity pool"
                    )
                };

            // Calculate the output amount of tokens based on the input amount 
            // and the pool fees
            let output_amount: Decimal = (output_tokens_vault.amount()
                * (dec!("1") - self.fee)
                * input_tokens.amount())
                / (input_tokens_vault.amount() + input_tokens.amount() 
                * (dec!("1") - self.fee));

            // Perform the swapping operation
            input_tokens_vault.put(input_tokens);
            output_tokens_vault.take(output_amount)
        }
        
        /// Adds liquidity to the liquidity pool
        pub fn add_liquidity(
            &mut self,
            bucket_a: Bucket,
            bucket_b: Bucket,
        ) -> (Bucket, Bucket, Bucket) {
            // Give the buckets the same names as the vaults
            let (mut bucket_a, mut bucket_b): (Bucket, Bucket) = 
            if bucket_a.resource_address()
                == self.vault_a.resource_address()
                && bucket_b.resource_address() == self.vault_b.resource_address()
            {
                (bucket_a, bucket_b)
            } else if bucket_a.resource_address() == self.vault_b.resource_address()
                && bucket_b.resource_address() == self.vault_a.resource_address()
            {
                (bucket_b, bucket_a)
            } else {
                panic!("One of the tokens does not belong to the pool!")
            };

            // Getting the values of `dm` and `dn` based on the sorted buckets
            let dm: Decimal = bucket_a.amount();
            let dn: Decimal = bucket_b.amount();

            // Getting the values of m and n from the liquidity pool vaults
            let m: Decimal = self.vault_a.amount();
            let n: Decimal = self.vault_b.amount();

            // Calculate the amount of tokens which will be added to each one of 
            //the vaults
            let (amount_a, amount_b): (Decimal, Decimal) =
                if ((m == Decimal::zero()) | (n == Decimal::zero())) 
                    | ((m / n) == (dm / dn)) 
                {
                    // Case 1
                    (dm, dn)
                } else if (m / n) < (dm / dn) {
                    // Case 2
                    (dn * m / n, dn)
                } else {
                    // Case 3
                    (dm, dm * n / m)
                };

            // Depositing the amount of tokens calculated into the liquidity pool
            self.vault_a.put(bucket_a.take(amount_a));
            self.vault_b.put(bucket_b.take(amount_b));

            // Mint pool units tokens to the liquidity provider
            let pool_units_manager: ResourceManager = self.pool_units_resource_manager;
            
            let pool_units_amount: Decimal =
                if pool_units_manager.total_supply().unwrap() == Decimal::zero() {
                    dec!("100.00")
                } else {
                    amount_a * pool_units_manager.total_supply().unwrap() / m
                };
            let pool_units: Bucket = pool_units_manager.mint(pool_units_amount);

            // Return the remaining tokens to the caller as well as the pool units 
            // tokens
            (bucket_a, bucket_b, pool_units)
        }

        /// Removes the amount of funds from the pool corresponding to the pool units.
        pub fn remove_liquidity(&mut self, pool_units: Bucket) -> (Bucket, Bucket) {
            assert!(
                pool_units.resource_address() == self.pool_units_resource_manager.resource_address(),
                "Wrong token type passed in"
            );

            // Get the resource manager of the lp tokens
            let pool_units_resource_manager: ResourceManager =
                self.pool_units_resource_manager;

            // Calculate the share based on the input LP tokens.
            let share = pool_units.amount() / 
                pool_units_resource_manager.total_supply().unwrap();

            // Burn the LP tokens received
            pool_units.burn();
    
            // Return the withdrawn tokens
            (
                self.vault_a.take(self.vault_a.amount() * share),
                self.vault_b.take(self.vault_b.amount() * share),
            )
        }

        pub fn get_swap_fee(&self) -> Decimal {
            return self.fee;
        }
    }
}