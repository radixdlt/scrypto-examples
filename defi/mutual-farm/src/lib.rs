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

import! {
r#"
{
    "package_address": "015d39c9a28c2ab646facfa9a7d303b1c9c7cf300611094a3ccc68",
    "blueprint_name": "PriceOracle",
    "functions": [
      {
        "name": "instantiate_oracle",
        "inputs": [
          {
            "type": "U32"
          }
        ],
        "output": {
          "type": "Tuple",
          "elements": [
            {
              "type": "Custom",
              "name": "Bucket",
              "generics": []
            },
            {
              "type": "Custom",
              "name": "ComponentAddress",
              "generics": []
            }
          ]
        }
      }
    ],
    "methods": [
      {
        "name": "get_price",
        "mutability": "Immutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "ResourceAddress",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "ResourceAddress",
            "generics": []
          }
        ],
        "output": {
          "type": "Option",
          "value": {
            "type": "Custom",
            "name": "Decimal",
            "generics": []
          }
        }
      },
      {
        "name": "update_price",
        "mutability": "Immutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "ResourceAddress",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "ResourceAddress",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "Decimal",
            "generics": []
          }
        ],
        "output": {
          "type": "Unit"
        }
      },
      {
        "name": "admin_badge_address",
        "mutability": "Immutable",
        "inputs": [],
        "output": {
          "type": "Custom",
          "name": "ResourceAddress",
          "generics": []
        }
      }
    ]
  }
"#
}

import! {
r#"
{
    "package_address": "01d897468d09529e7b6b72bafa726670e95cd1f19e08ad6deaa1f2",
    "blueprint_name": "SyntheticPool",
    "functions": [
      {
        "name": "instantiate_pool",
        "inputs": [
          {
            "type": "Custom",
            "name": "ComponentAddress",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "ResourceAddress",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "ResourceAddress",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "Decimal",
            "generics": []
          }
        ],
        "output": {
          "type": "Custom",
          "name": "ComponentAddress",
          "generics": []
        }
      }
    ],
    "methods": [
      {
        "name": "add_synthetic_token",
        "mutability": "Mutable",
        "inputs": [
          {
            "type": "String"
          },
          {
            "type": "Custom",
            "name": "ResourceAddress",
            "generics": []
          }
        ],
        "output": {
          "type": "Custom",
          "name": "ResourceAddress",
          "generics": []
        }
      },
      {
        "name": "stake",
        "mutability": "Mutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "Proof",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "Bucket",
            "generics": []
          }
        ],
        "output": {
          "type": "Unit"
        }
      },
      {
        "name": "unstake",
        "mutability": "Mutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "Proof",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "Decimal",
            "generics": []
          }
        ],
        "output": {
          "type": "Custom",
          "name": "Bucket",
          "generics": []
        }
      },
      {
        "name": "mint",
        "mutability": "Mutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "Proof",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "Decimal",
            "generics": []
          },
          {
            "type": "String"
          }
        ],
        "output": {
          "type": "Custom",
          "name": "Bucket",
          "generics": []
        }
      },
      {
        "name": "burn",
        "mutability": "Mutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "Proof",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "Bucket",
            "generics": []
          }
        ],
        "output": {
          "type": "Unit"
        }
      },
      {
        "name": "get_total_global_debt",
        "mutability": "Immutable",
        "inputs": [],
        "output": {
          "type": "Custom",
          "name": "Decimal",
          "generics": []
        }
      },
      {
        "name": "get_snx_price",
        "mutability": "Immutable",
        "inputs": [],
        "output": {
          "type": "Custom",
          "name": "Decimal",
          "generics": []
        }
      },
      {
        "name": "get_asset_price",
        "mutability": "Immutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "ResourceAddress",
            "generics": []
          }
        ],
        "output": {
          "type": "Custom",
          "name": "Decimal",
          "generics": []
        }
      },
      {
        "name": "get_user_summary",
        "mutability": "Mutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "ResourceAddress",
            "generics": []
          }
        ],
        "output": {
          "type": "String"
        }
      },
      {
        "name": "new_user",
        "mutability": "Immutable",
        "inputs": [],
        "output": {
          "type": "Custom",
          "name": "Bucket",
          "generics": []
        }
      }
    ]
  }
"#
}

import! {
r#"
{
    "package_address": "0107467fe140289bec61d5ec6be68d6c7adc88ea5528d90c28b9f5",
    "blueprint_name": "Radiswap",
    "functions": [
      {
        "name": "instantiate_pool",
        "inputs": [
          {
            "type": "Custom",
            "name": "Bucket",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "Bucket",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "Decimal",
            "generics": []
          },
          {
            "type": "String"
          },
          {
            "type": "String"
          },
          {
            "type": "String"
          },
          {
            "type": "Custom",
            "name": "Decimal",
            "generics": []
          }
        ],
        "output": {
          "type": "Tuple",
          "elements": [
            {
              "type": "Custom",
              "name": "ComponentAddress",
              "generics": []
            },
            {
              "type": "Custom",
              "name": "Bucket",
              "generics": []
            }
          ]
        }
      }
    ],
    "methods": [
      {
        "name": "add_liquidity",
        "mutability": "Mutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "Bucket",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "Bucket",
            "generics": []
          }
        ],
        "output": {
          "type": "Tuple",
          "elements": [
            {
              "type": "Custom",
              "name": "Bucket",
              "generics": []
            },
            {
              "type": "Custom",
              "name": "Bucket",
              "generics": []
            }
          ]
        }
      },
      {
        "name": "remove_liquidity",
        "mutability": "Mutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "Bucket",
            "generics": []
          }
        ],
        "output": {
          "type": "Tuple",
          "elements": [
            {
              "type": "Custom",
              "name": "Bucket",
              "generics": []
            },
            {
              "type": "Custom",
              "name": "Bucket",
              "generics": []
            }
          ]
        }
      },
      {
        "name": "swap",
        "mutability": "Mutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "Bucket",
            "generics": []
          }
        ],
        "output": {
          "type": "Custom",
          "name": "Bucket",
          "generics": []
        }
      },
      {
        "name": "get_pair",
        "mutability": "Immutable",
        "inputs": [],
        "output": {
          "type": "Tuple",
          "elements": [
            {
              "type": "Custom",
              "name": "ResourceAddress",
              "generics": []
            },
            {
              "type": "Custom",
              "name": "ResourceAddress",
              "generics": []
            }
          ]
        }
      }
    ]
  }
"#
}

blueprint! {
    struct MutualFarm {
        /// Badge for interacting with other components.
        identity_badge: Vault,
        /// XRD/SNX Radiswap
        xrd_snx_radiswap: Radiswap,
        /// Price Oracle
        price_oracle: PriceOracle,
        /// Synthetic for minting synthetic tokens
        synthetic_pool: SyntheticPool,

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
        radiswap: Radiswap,
        /// Radiswap LP token vault
        radiswap_lp_tokens: Vault,

        /// Mutual farm share resource address
        mutual_farm_share_resource_address: ResourceAddress,
        /// Total contribution
        total_contribution_in_usd: Decimal,
    }

    impl MutualFarm {
        pub fn instantiate_farm(
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
            let price_oracle: PriceOracle = price_oracle_address.into();
            let xrd_usd_price = price_oracle
                .get_price(initial_xrd.resource_address(), usd_address)
                .unwrap();
            let snx_usd_price = price_oracle.get_price(snx_address, usd_address).unwrap();
            let tesla_usd_price = price_oracle.get_price(asset_address, usd_address).unwrap();

            debug!("Swap 3/4 of XRD for SNX");
            let xrd_snx_radiswap: Radiswap = xrd_snx_radiswap_address.into();
            let xrd_amount = initial_xrd.amount();
            let snx = xrd_snx_radiswap.swap(initial_xrd.take(initial_xrd.amount() * 3 / 4));
            let snx_amount = snx.amount();

            debug!("Deposit SNX into synthetic pool and mint sTESLA (1/10 of our SNX).");
            let price_oracle: PriceOracle = price_oracle_address.into();
            let synthetic_pool: SyntheticPool = synthetic_pool_address.into();
            synthetic_pool.add_synthetic_token(asset_symbol.clone(), asset_address);
            synthetic_pool.stake(identity_badge.create_proof(), snx);
            
            let quantity = snx_amount * snx_usd_price / 10 / tesla_usd_price;
            let synth =
                synthetic_pool.mint(identity_badge.create_proof(), quantity, asset_symbol.clone());
            let synth_address = synth.resource_address();

            debug!("Set up sTESLA/XRD swap pool");
            let (radiswap_comp, lp_tokens) = Radiswap::instantiate_pool(
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
            let snx = self.xrd_snx_radiswap.swap(xrd.take(xrd.amount() * dec!("3") / dec!("4")));
            let snx_amount = snx.amount();

            debug!("Deposit SNX into synthetic pool and mint sTESLA (1/10 of our SNX).");
            self.synthetic_pool.stake(self.identity_badge.create_proof(), snx);
            let quantity = snx_amount * snx_usd_price / dec!("10") / tesla_usd_price;
            let synth = self.synthetic_pool.mint(self.identity_badge.create_proof(), quantity, self.asset_symbol.clone());

            debug!("Add liquidity to sTESLA/XRD swap pool");
            let (lp_tokens, mut remainder) = self.radiswap.add_liquidity(synth, xrd);
            if remainder.resource_address() == self.synth_address {
                self.synthetic_pool.burn(self.identity_badge.create_proof(), remainder);
                remainder = Bucket::new(xrd_address);
            }
            self.radiswap_lp_tokens.put(lp_tokens);

            debug!("Mint initial shares");
            let contribution = xrd_usd_price * (xrd_amount - remainder.amount());
            let num_shares_to_issue = contribution
                / (self.total_contribution_in_usd / borrow_resource_manager!(self.mutual_farm_share_resource_address).total_supply());
            self.total_contribution_in_usd += contribution;
            let shares = self.identity_badge.authorize(|| {
                borrow_resource_manager!(self.mutual_farm_share_resource_address).mint(num_shares_to_issue)
            });
            (shares, remainder)
        }

        pub fn withdraw(&mut self) -> (Bucket, Bucket) {
            todo!()
        }
    }
}
