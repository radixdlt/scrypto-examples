use scrypto::prelude::*;

#[blueprint]
mod resource_creation {
    struct ResourceCreation {
        vaults: KeyValueStore<ResourceAddress, Vault>,
        all_auth_resources: Vec<ResourceAddress>,
        auth_vault_alpha: Vault,
        auth_vault_bravo: Vault,
        auth_vault_charlie: Vault,
    }

    impl ResourceCreation {
        pub fn instantiate() -> ComponentAddress {
            let alpha =
                Vault::with_bucket(ResourceCreation::create_basic_badge("Alpha".to_owned()));
            let bravo =
                Vault::with_bucket(ResourceCreation::create_basic_badge("Bravo".to_owned()));
            let charlie =
                Vault::with_bucket(ResourceCreation::create_basic_badge("Charlie".to_owned()));
            let authorities: Vec<ResourceAddress> = vec![
                alpha.resource_address(),
                bravo.resource_address(),
                charlie.resource_address(),
            ];

            Self {
                vaults: KeyValueStore::new(),
                all_auth_resources: authorities,
                auth_vault_alpha: alpha,
                auth_vault_bravo: bravo,
                auth_vault_charlie: charlie,
            }
            .instantiate()
            .globalize()
        }

        pub fn create_basic_badge(name: String) -> Bucket {
            ResourceBuilder::new_fungible()
                .metadata("name", format!("{} authority token", name))
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1)
        }

        /// Examples of creating very basic fungibles
        pub fn create_basic_fungibles(&mut self) {
            // We will provide a "name" metadata field for each one, to ease tracking
            // This is not a requirement - a resource may be created with no additional metadata

            let bucket_1 = ResourceBuilder::new_fungible()
                .metadata("name", "Fixed supply")
                .mint_initial_supply(101);
            let resource_address_1 = bucket_1.resource_address();
            let vault_1 = Vault::with_bucket(bucket_1);
            self.vaults.insert(resource_address_1, vault_1);

            let bucket_2 = ResourceBuilder::new_fungible()
                .metadata("name", "Fixed supply, indivisible")
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(102);
            let resource_address_2 = bucket_2.resource_address();
            let vault_2 = Vault::with_bucket(bucket_2);
            self.vaults.insert(resource_address_2, vault_2);

            let bucket_3 = ResourceBuilder::new_fungible()
                .metadata("name", "Mutable supply, single mint/burn authority")
                .mintable(
                    rule!(require(self.auth_vault_alpha.resource_address())),
                    LOCKED,
                )
                .burnable(
                    rule!(require(self.auth_vault_alpha.resource_address())),
                    LOCKED,
                )
                .mint_initial_supply(103);
            let resource_address_3 = bucket_3.resource_address();
            let vault_3 = Vault::with_bucket(bucket_3);
            self.vaults.insert(resource_address_3, vault_3);

            let bucket_4 = ResourceBuilder::new_fungible()
                .metadata(
                    "name",
                    "Mutable supply, single mint authority, burnable by any holder",
                )
                .mintable(
                    rule!(require(self.auth_vault_alpha.resource_address())),
                    LOCKED,
                )
                .burnable(rule!(allow_all), LOCKED)
                .mint_initial_supply(104);
            let resource_address_4 = bucket_4.resource_address();
            let vault_4 = Vault::with_bucket(bucket_4);
            self.vaults.insert(resource_address_4, vault_4);

            let bucket_5 = ResourceBuilder::new_fungible()
                .metadata(
                    "name",
                    "Mutable supply, mintable by 2-of-3 admins, can not be burned",
                )
                .mintable(
                    rule!(require_n_of(2, self.all_auth_resources.clone())),
                    LOCKED,
                )
                .mint_initial_supply(105);
            let resource_address_5 = bucket_5.resource_address();
            let vault_5 = Vault::with_bucket(bucket_5);
            self.vaults.insert(resource_address_5, vault_5);
        }

        /// Create the resource, and then mint as a separate action
        pub fn create_and_mint_as_separate_actions(&mut self) {
            let resource_address = ResourceBuilder::new_fungible()
                .metadata("name", "Mintable token, any admin able to mint")
                .mintable(
                    rule!(require_any_of(self.all_auth_resources.clone())),
                    LOCKED,
                )
                .create_with_no_initial_supply();

            let resource_manager = borrow_resource_manager!(resource_address);

            // Mint method 1 - put one of our auth tokens into the local authorization zone and mint
            ComponentAuthZone::push(self.auth_vault_charlie.create_proof());
            let mut bucket_1 = resource_manager.mint(10);
            ComponentAuthZone::pop().drop();

            // Mint method 2 - use the convenience "authorize" method on Vault to accomplish the same
            let bucket_2 = self
                .auth_vault_charlie
                .authorize(|| resource_manager.mint(20));

            // Combine our two buckets and then store in a vault
            bucket_1.put(bucket_2);
            let new_vault = Vault::with_bucket(bucket_1);
            self.vaults.insert(resource_address, new_vault);
        }

        /// Create a token which can only be deposited, never withdrawn
        pub fn create_restricted_transfer_token(&mut self) -> Bucket {
            // Deposit-only is a good choice for something like a regulated token which
            // you can't have users freely passing around
            ResourceBuilder::new_fungible()
                .metadata(
                    "name",
                    "Token which can not be withdrawn from a vault once stored",
                )
                .restrict_withdraw(rule!(deny_all), LOCKED)
                .mint_initial_supply(1)
        }

        /// Create a token with updateable metadata
        pub fn create_token_with_mutable_metadata(&mut self) {
            let bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Updateable metadata token")
                .updateable_metadata(
                    rule!(require(self.auth_vault_alpha.resource_address())),
                    LOCKED,
                )
                .mint_initial_supply(100);
            let resource_address = bucket.resource_address();
            let vault = Vault::with_bucket(bucket);
            self.vaults.insert(resource_address, vault);

            // Example of how to update the metadata
            // You can overwrite an old value, or create a new key, or both, in a single action
            let resource_manager = borrow_resource_manager!(resource_address);
            self.auth_vault_alpha.authorize(|| {
                resource_manager.metadata().set("name".to_owned(), "An even better name".to_owned());
                resource_manager
                    .metadata()
                    .set("some new key".to_owned(), "Interesting value".to_owned());
            });
        }

        /// Create a freezable token
        pub fn create_freezable_token(&mut self) {
            // Start the token in a freely-transferrable state, with the alpha badge able to change the rules
            let bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Freezable token")
                .restrict_deposit(
                    rule!(allow_all),
                    MUTABLE(rule!(require(self.auth_vault_alpha.resource_address()))),
                )
                .restrict_withdraw(
                    rule!(allow_all),
                    MUTABLE(rule!(require(self.auth_vault_alpha.resource_address()))),
                )
                .mint_initial_supply(100);
            let resource_address = bucket.resource_address();
            let vault = Vault::with_bucket(bucket);
            self.vaults.insert(resource_address.clone(), vault);

            // Put our admin badge in local auth zone, so we can use it for privileged actions
            ComponentAuthZone::push(self.auth_vault_alpha.create_proof());

            // Freeze the token, so no one may withdraw or deposit it
            let resource_manager = borrow_resource_manager!(resource_address);
            resource_manager.set_depositable(rule!(deny_all));
            resource_manager.set_withdrawable(rule!(deny_all));

            // Unfreeze the token!
            resource_manager.set_depositable(rule!(allow_all));
            resource_manager.set_withdrawable(rule!(allow_all));

            // Lock the token in the unfrozen state, so it may never again be changed
            resource_manager.lock_depositable();
            resource_manager.lock_withdrawable();
        }

        /// Withdraws all of specified resource
        pub fn withdraw_resource(&mut self, resource_address: ResourceAddress) -> Bucket {
            let vault = self.vaults.get_mut(&resource_address);
            match vault {
                Some(mut vault) => vault.take_all(),
                None => {
                    panic!("No such resource present");
                }
            }
        }

        /// Withdraws all authority badges
        pub fn withdraw_admin_badges(&mut self) -> (Bucket, Bucket, Bucket) {
            (
                self.auth_vault_alpha.take_all(),
                self.auth_vault_bravo.take_all(),
                self.auth_vault_charlie.take_all(),
            )
        }
    }
}
