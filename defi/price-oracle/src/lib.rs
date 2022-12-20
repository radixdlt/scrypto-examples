use scrypto::prelude::*;

blueprint! {
    struct PriceOracle {
        /// Last price of each resource pair
        prices: KeyValueStore<(ResourceAddress, ResourceAddress), Decimal>,
        /// The admin badge resource def address
        admin_badge: ResourceAddress,
    }

    impl PriceOracle {
        /// Creates a PriceOracle component, along with admin badges.
        pub fn instantiate_oracle(num_of_admins: u32) -> (Bucket, ComponentAddress) {
            assert!(num_of_admins >= 1);

            let badges = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Price Oracle Admin Badge")
                .initial_supply(num_of_admins);

            let rules = AccessRules::new()
                .method("update_price", rule!(require(badges.resource_address())), AccessRule::DenyAll)
                .default(rule!(allow_all), AccessRule::DenyAll);

            let mut component = Self {
                prices: KeyValueStore::new(),
                admin_badge: badges.resource_address(),
            }
            .instantiate();
            component.add_access_check(rules);
            let component_address = component.globalize();

            (badges, component_address)
        }

        /// Returns the current price of a resource pair BASE/QUOTE.
        pub fn get_price(&self, base: ResourceAddress, quote: ResourceAddress) -> Option<Decimal> {
            match self.prices.get(&(base, quote)) {
                Some(price) => Some(*price),
                None => None,
            }
        }

        /// Updates the price of a resource pair BASE/QUOTE and its inverse.
        pub fn update_price(&self, base: ResourceAddress, quote: ResourceAddress, price: Decimal) {
            self.prices.insert((base, quote), price);
            self.prices.insert((quote, base), dec!("1") / price);
        }

        /// Returns the admin badge resource address.
        pub fn admin_badge_address(&self) -> ResourceAddress {
            self.admin_badge
        }
    }
}
