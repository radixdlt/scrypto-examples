use scrypto::prelude::*;

#[blueprint]
mod radiswap_module {
    struct Radiswap {
        /// A vault containing pool reverses of reserves of token A.
        vault_a: Vault,
        /// A vault containing pool reverses of reserves of token B.
        vault_b: Vault,
        /// The token address of a token representing pool units in this pool
        pool_units_resource_address: ResourceAddress,
        /// A vault containing a badge which has the authority to mint `pool_units` 
        /// tokens.
        pool_units_minter_badge: Vault,
        /// The amount of fees imposed by the pool on swaps where 0 <= fee <= 1.
        fee: Decimal,
    }

    impl Radiswap {
        /// Creates a new liquidity pool of the two tokens sent to the pool
        pub fn instantiate_radiswap(
            bucket_a: Bucket,
            bucket_b: Bucket,
            fee: Decimal,
        ) -> (ComponentAddress, Bucket) {
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

            // Create a badge which will be given the authority to mint the pool  
            // unit tokens.
            let pool_units_minter_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "LP Token Mint Auth")
                .mint_initial_supply(1);

            // Create the pool units token along with the initial supply specified  
            // by the user.
            let pool_units: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "Pool Unit")
                .metadata("symbol", "UNIT")
                .mintable(
                    rule!(require(pool_units_minter_badge.resource_address())),
                    LOCKED,
                )
                .burnable(
                    rule!(require(pool_units_minter_badge.resource_address())),
                    LOCKED,
                )
                .mint_initial_supply(100);

            let access_rules_config = AccessRulesConfig::new()
                .default(rule!(allow_all), rule!(deny_all));

            // Create the Radiswap component and globalize it
            let radiswap: ComponentAddress = Self {
                vault_a: Vault::with_bucket(bucket_a),
                vault_b: Vault::with_bucket(bucket_b),
                pool_units_resource_address: pool_units.resource_address(),
                pool_units_minter_badge: Vault::with_bucket(pool_units_minter_badge),
                fee: fee,
            }
            .instantiate()
            .globalize_with_access_rules(access_rules_config);
            
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
            let pool_units_manager: ResourceManager =
                borrow_resource_manager!(self.pool_units_resource_address);
            let pool_units_amount: Decimal =
                if pool_units_manager.total_supply() == Decimal::zero() {
                    dec!("100.00")
                } else {
                    amount_a * pool_units_manager.total_supply() / m
                };
            let pool_units: Bucket = self
                .pool_units_minter_badge
                .authorize(|| pool_units_manager.mint(pool_units_amount));

            // Return the remaining tokens to the caller as well as the pool units 
            // tokens
            (bucket_a, bucket_b, pool_units)
        }

        /// Removes the amount of funds from the pool corresponding to the pool units.
        pub fn remove_liquidity(&mut self, pool_units: Bucket) -> (Bucket, Bucket) {
            assert!(
                pool_units.resource_address() == self.pool_units_resource_address,
                "Wrong token type passed in"
            );

            // Get the resource manager of the lp tokens
            let pool_units_resource_manager: ResourceManager =
                borrow_resource_manager!(self.pool_units_resource_address);

            // Calculate the share based on the input LP tokens.
            let share = pool_units.amount() / 
                pool_units_resource_manager.total_supply();

            // Burn the LP tokens received
            self.pool_units_minter_badge.authorize(|| {
                pool_units.burn();
            });

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