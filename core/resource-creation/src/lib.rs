use scrypto::prelude::*;

blueprint! {
    struct ResourceCreation {        
        vaults: LazyMap<ResourceAddress, Vault>,
        all_auth_resources: Vec<ResourceAddress>,
        auth_vault_alpha: Vault,
        auth_vault_bravo: Vault,
        auth_vault_charlie: Vault,
    }

    impl ResourceCreation {

        pub fn instantiate() -> ComponentAddress {
            let alpha = Vault::with_bucket(ResourceCreation::create_basic_badge(String::from("Alpha")));
            let bravo = Vault::with_bucket(ResourceCreation::create_basic_badge(String::from("Bravo")));
            let charlie = Vault::with_bucket(ResourceCreation::create_basic_badge(String::from("Charlie")));
            let authorities: Vec<ResourceAddress> = vec![alpha.resource_address(), bravo.resource_address(), charlie.resource_address()];

            Self {
                vaults: LazyMap::new(),
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
                .initial_supply(1)
        }

        /// Examples of creating very basic fungibles
        pub fn create_basic_fungibles(&mut self) {
            // We will provide a "name" metadata field for each one, to ease tracking
            // This is not a requirement - a resource may be created with no additional metadata

            let bucket_1 = ResourceBuilder::new_fungible()
                .metadata("name", "Fixed supply")
                .initial_supply(101);
            let resource_address_1 = bucket_1.resource_address();
            let vault_1 = Vault::with_bucket(bucket_1);
            self.vaults.insert(resource_address_1, vault_1);

            let bucket_2 = ResourceBuilder::new_fungible()
                .metadata("name", "Fixed supply, indivisible")
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(102);
            let resource_address_2 = bucket_2.resource_address();
            let vault_2 = Vault::with_bucket(bucket_2);
            self.vaults.insert(resource_address_2, vault_2);

            let bucket_3 = ResourceBuilder::new_fungible()
                .metadata("name", "Mutable supply, single mint/burn authority")
                .mintable(rule!(require(self.auth_vault_alpha.resource_address())), LOCKED)
                .burnable(rule!(require(self.auth_vault_alpha.resource_address())), LOCKED)
                .initial_supply(dec!(103));
            let resource_address_3 = bucket_3.resource_address();
            let vault_3 = Vault::with_bucket(bucket_3);
            self.vaults.insert(resource_address_3, vault_3);
            
            let bucket_4 = ResourceBuilder::new_fungible()
                .metadata("name", "Mutable supply, single mint authority, burnable by any holder")
                .mintable(rule!(require(self.auth_vault_alpha.resource_address())), LOCKED)
                .burnable(rule!(allow_all), LOCKED)
                .initial_supply(dec!(104));
            let resource_address_4 = bucket_4.resource_address();
            let vault_4 = Vault::with_bucket(bucket_4);
            self.vaults.insert(resource_address_4, vault_4);
            
            let bucket_5 = ResourceBuilder::new_fungible()
                .metadata("name", "Mutable supply, mintable by 2-of-3 admins, can not be burned")
                .mintable(rule!(require_n_of(2, "all_auth_resources")), LOCKED)
                .initial_supply(dec!(105));
            let resource_address_5 = bucket_5.resource_address();
            let vault_5 = Vault::with_bucket(bucket_5);
            self.vaults.insert(resource_address_5, vault_5);
        }

        /// Withdraws all of specified resource
        pub fn withdraw_resource(&mut self, resource_address: ResourceAddress) -> Bucket {
            let vault = self.vaults.get(&resource_address);
            match vault {
                Some(mut vault) => vault.take_all(),
                None => {
                    panic!("No such resource present");
                }
            }
        }

        /// Withdraws all authority badges
        pub fn withdraw_admin_badges(&mut self) -> (Bucket, Bucket, Bucket) {
            (self.auth_vault_alpha.take_all(), self.auth_vault_bravo.take_all(), self.auth_vault_charlie.take_all())
        }
        
    }
}