use scrypto::prelude::*;

// Welcome to MutualFund!
//
// Start earning today by converting your XRD into liquidity.
//
// For every 1 XRD invested,
// 1. We immediately convert 0.75 XRD into SNX
// 2. All SNX will be staked into a Synthetic Pool
// 3. We mint Synthetic TESLA token a 1000% collateralization ratio
// dec!("4"). The minted sTELSA and 0.25 XRD will be added to a sTESLA/XRD swap pool owned by us (with change returned to you)
// 5. Based on your contribution (in dollar amount), we issue MutualFund share tokens which allow you to redeem underlying assets and claim dividends.

external_blueprint! {
  PriceOraclePackageTarget {
    fn instantiate_oracle(num_of_admins: u32) -> (Bucket, ComponentAddress);
    
  }
}

external_component! {
    PriceOracleComponentTarget {
        fn get_price(&self, base: ResourceAddress, quote: ResourceAddress) -> Option<Decimal>;
        fn update_price(&self, base: ResourceAddress, quote: ResourceAddress, price: Decimal);
        fn admin_badge_address(&self) -> ResourceAddress;
    }
}

external_blueprint! {
  SyntheticPoolPackageTarget {
    fn instantiate_pool(oracle_address: ComponentAddress, snx_token_address: ResourceAddress, usd_token_address: ResourceAddress, collateralization_threshold: Decimal) -> ComponentAddress;
  }
}

external_component! {
    SyntheticPoolComponentTarget {
        fn add_synthetic_token(&mut self,asset_symbol: String,asset_address: ResourceAddress) -> ResourceAddress;
        fn stake(&mut self, user_auth: Proof, stake_in_snx: Bucket);
        fn unstake(&mut self, user_auth: Proof, amount: Decimal) -> Bucket;
        fn mint(&mut self, user_auth: Proof, amount: Decimal, symbol: String) -> Bucket;
        fn burn(&mut self, user_auth: Proof, bucket: Bucket);
        fn get_total_global_debt(&self) -> Decimal;
        fn get_snx_price(&self) -> Decimal;
        fn get_asset_price(&self, asset_address: ResourceAddress) -> Decimal;
        fn get_user_summary(&mut self, user_id: ResourceAddress) -> String;
        fn new_user(&self) -> Bucket;
    }
}

external_blueprint! {
  RadiswapPackageTarget {
    fn instantiate_pool(a_tokens: Bucket, b_tokens: Bucket, lp_initial_supply: Decimal, lp_symbol: String, lp_name: String, lp_url: String, fee: Decimal) -> (ComponentAddress, Bucket);
  }
}

external_component! {
    RadiswapComponentTarget {
        fn add_liquidity(&mut self, a_tokens: Bucket, b_tokens: Bucket) -> (Bucket, Bucket);
        fn remove_liquidity(&mut self, lp_tokens: Bucket) -> (Bucket, Bucket);
        fn swap(&mut self, input_tokens: Bucket) -> Bucket;
        fn get_pair(&self) -> (ResourceAddress, ResourceAddress);
      }
}

#[blueprint]
mod mutual_farm {
    struct MutualFarm {
        /// Badge for interacting with other components.
        identity_badge: Vault,
        /// XRD/SNX Radiswap
        xrd_snx_radiswap: RadiswapComponentTarget,
        /// Price Oracle
        price_oracle: PriceOracleComponentTarget,
        /// Synthetic for minting synthetic tokens
        synthetic_pool: SyntheticPoolComponentTarget,

        /// Asset symbol
        asset_symbol: String,
        /// Asset address
        asset_address: ResourceAddress,
        /// Synthetic asset address
        synth_address: ResourceAddress,
        /// SNX resource address
        snx_address: ResourceAddress,
        /// USD resource address
        usd_address: ResourceAddress,

        /// Radiswap for sTESLA/XRD
        radiswap: RadiswapComponentTarget,
        /// Radiswap LP token vault
        radiswap_lp_tokens: Vault,

        /// Mutual farm share resource address
        mutual_farm_share_resource_address: ResourceAddress,
        /// Total contribution
        total_contribution_in_usd: Decimal,
    }

    impl MutualFarm {
        pub fn instantiate_farm(
            radiswap_package_address: PackageAddress,
            price_oracle_address: ComponentAddress,
            xrd_snx_radiswap_address: ComponentAddress,
            synthetic_pool_address: ComponentAddress,
            asset_symbol: String,
            asset_address: ResourceAddress,
            initial_shares: Decimal,
            mut initial_xrd: Bucket,
            snx_address: ResourceAddress,
            usd_address: ResourceAddress,
        ) -> (Bucket, ComponentAddress) {
            debug!("Create an identity badge for accessing other components");
            let identity_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "ID")
                .initial_supply(1);
            let identity_badge_address = identity_badge.resource_address();

            debug!("Fetch price info from oracle");
            let price_oracle: PriceOracleComponentTarget = price_oracle_address.into();
            let xrd_usd_price = price_oracle
                .get_price(initial_xrd.resource_address(), usd_address)
                .unwrap();
            let snx_usd_price = price_oracle.get_price(snx_address, usd_address).unwrap();
            let tesla_usd_price = price_oracle.get_price(asset_address, usd_address).unwrap();

            debug!("Swap 3/4 of XRD for SNX");
            let mut xrd_snx_radiswap: RadiswapComponentTarget = xrd_snx_radiswap_address.into();
            let xrd_amount = initial_xrd.amount();
            let snx = xrd_snx_radiswap.swap(initial_xrd.take(initial_xrd.amount() * 3 / 4));
            let snx_amount = snx.amount();

            debug!("Deposit SNX into synthetic pool and mint sTESLA (1/10 of our SNX).");
            let price_oracle: PriceOracleComponentTarget = price_oracle_address.into();
            let mut synthetic_pool: SyntheticPoolComponentTarget = synthetic_pool_address.into();
            synthetic_pool.add_synthetic_token(asset_symbol.clone(), asset_address);
            synthetic_pool.stake(identity_badge.create_proof(), snx);

            let quantity = snx_amount * snx_usd_price / 10 / tesla_usd_price;
            let synth = synthetic_pool.mint(
                identity_badge.create_proof(),
                quantity,
                asset_symbol.clone(),
            );
            let synth_address = synth.resource_address();

            debug!("Set up sTESLA/XRD swap pool");
            let (radiswap_comp, lp_tokens) = RadiswapPackageTarget::at(radiswap_package_address, "Radiswap").instantiate_pool(
                synth,
                initial_xrd,
                dec!("1000000"),
                "LP".to_owned(),
                "LP Token".to_owned(),
                "https://example.com/".to_owned(),
                "0.003".parse().unwrap(),
            );

            debug!("Mint initial shares");
            let mutual_farm_share_resource_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "MutualFarm share")
                .mintable(rule!(require(identity_badge_address)), LOCKED)
                .burnable(rule!(require(identity_badge_address)), LOCKED)
                .no_initial_supply();
            let shares = identity_badge.authorize(|| {
                borrow_resource_manager!(mutual_farm_share_resource_address).mint(initial_shares)
            });

            debug!("Instantiate MutualFund component");
            let component = Self {
                identity_badge: Vault::with_bucket(identity_badge),
                price_oracle,
                xrd_snx_radiswap,
                synthetic_pool,
                asset_symbol,
                asset_address,
                synth_address,
                snx_address,
                usd_address,
                radiswap: radiswap_comp.into(),
                radiswap_lp_tokens: Vault::with_bucket(lp_tokens),
                mutual_farm_share_resource_address,
                total_contribution_in_usd: xrd_amount * xrd_usd_price,
            }
            .instantiate()
            .globalize();

            (shares, component)
        }

        pub fn deposit(&mut self, mut xrd: Bucket) -> (Bucket, Bucket) {
            debug!("Fetch price info from oracle");
            let xrd_usd_price = self
                .price_oracle
                .get_price(xrd.resource_address(), self.usd_address)
                .unwrap();
            let snx_usd_price = self
                .price_oracle
                .get_price(self.snx_address, self.usd_address)
                .unwrap();
            let tesla_usd_price = self
                .price_oracle
                .get_price(self.asset_address, self.usd_address)
                .unwrap();

            debug!("Swap 3/4 of XRD for SNX");
            let xrd_address = xrd.resource_address();
            let xrd_amount = xrd.amount();
            let snx = self
                .xrd_snx_radiswap
                .swap(xrd.take(xrd.amount() * dec!("3") / dec!("4")));
            let snx_amount = snx.amount();

            debug!("Deposit SNX into synthetic pool and mint sTESLA (1/10 of our SNX).");
            self.synthetic_pool
                .stake(self.identity_badge.create_proof(), snx);
            let quantity = snx_amount * snx_usd_price / dec!("10") / tesla_usd_price;
            let synth = self.synthetic_pool.mint(
                self.identity_badge.create_proof(),
                quantity,
                self.asset_symbol.clone(),
            );

            debug!("Add liquidity to sTESLA/XRD swap pool");
            let (lp_tokens, mut remainder) = self.radiswap.add_liquidity(synth, xrd);
            if remainder.resource_address() == self.synth_address {
                self.synthetic_pool
                    .burn(self.identity_badge.create_proof(), remainder);
                remainder = Bucket::new(xrd_address);
            }
            self.radiswap_lp_tokens.put(lp_tokens);

            debug!("Mint initial shares");
            let contribution = xrd_usd_price * (xrd_amount - remainder.amount());
            let num_shares_to_issue = contribution
                / (self.total_contribution_in_usd
                    / borrow_resource_manager!(self.mutual_farm_share_resource_address)
                        .total_supply());
            self.total_contribution_in_usd += contribution;
            let shares = self.identity_badge.authorize(|| {
                borrow_resource_manager!(self.mutual_farm_share_resource_address)
                    .mint(num_shares_to_issue)
            });
            (shares, remainder)
        }

        pub fn withdraw(&mut self) -> (Bucket, Bucket) {
            todo!()
        }
    }
}
