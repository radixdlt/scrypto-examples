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
      "package_address": "package_sim1q8pe9fczej7zhq4ty35q8uvf58h7wj45y3gufz539ysqm7l0ur",
      "blueprint_name": "PriceOracle",
      "abi": {
        "structure": {
          "type": "Struct",
          "name": "PriceOracle",
          "fields": {
            "type": "Named",
            "named": [
              [
                "prices",
                {
                  "type": "Custom",
                  "type_id": 131,
                  "generics": [
                    {
                      "type": "Tuple",
                      "elements": [
                        {
                          "type": "Custom",
                          "type_id": 182,
                          "generics": []
                        },
                        {
                          "type": "Custom",
                          "type_id": 182,
                          "generics": []
                        }
                      ]
                    },
                    {
                      "type": "Custom",
                      "type_id": 161,
                      "generics": []
                    }
                  ]
                }
              ],
              [
                "admin_badge",
                {
                  "type": "Custom",
                  "type_id": 182,
                  "generics": []
                }
              ]
            ]
          }
        },
        "fns": [
          {
            "ident": "instantiate_oracle",
            "mutability": null,
            "input": {
              "type": "Struct",
              "name": "PriceOracle_instantiate_oracle_Input",
              "fields": {
                "type": "Named",
                "named": [
                  [
                    "arg0",
                    {
                      "type": "U32"
                    }
                  ]
                ]
              }
            },
            "output": {
              "type": "Tuple",
              "elements": [
                {
                  "type": "Custom",
                  "type_id": 177,
                  "generics": []
                },
                {
                  "type": "Custom",
                  "type_id": 129,
                  "generics": []
                }
              ]
            },
            "export_name": "PriceOracle_instantiate_oracle"
          },
          {
            "ident": "get_price",
            "mutability": "Immutable",
            "input": {
              "type": "Struct",
              "name": "PriceOracle_get_price_Input",
              "fields": {
                "type": "Named",
                "named": [
                  [
                    "arg0",
                    {
                      "type": "Custom",
                      "type_id": 182,
                      "generics": []
                    }
                  ],
                  [
                    "arg1",
                    {
                      "type": "Custom",
                      "type_id": 182,
                      "generics": []
                    }
                  ]
                ]
              }
            },
            "output": {
              "type": "Option",
              "value": {
                "type": "Custom",
                "type_id": 161,
                "generics": []
              }
            },
            "export_name": "PriceOracle_get_price"
          },
          {
            "ident": "update_price",
            "mutability": "Immutable",
            "input": {
              "type": "Struct",
              "name": "PriceOracle_update_price_Input",
              "fields": {
                "type": "Named",
                "named": [
                  [
                    "arg0",
                    {
                      "type": "Custom",
                      "type_id": 182,
                      "generics": []
                    }
                  ],
                  [
                    "arg1",
                    {
                      "type": "Custom",
                      "type_id": 182,
                      "generics": []
                    }
                  ],
                  [
                    "arg2",
                    {
                      "type": "Custom",
                      "type_id": 161,
                      "generics": []
                    }
                  ]
                ]
              }
            },
            "output": {
              "type": "Unit"
            },
            "export_name": "PriceOracle_update_price"
          },
          {
            "ident": "admin_badge_address",
            "mutability": "Immutable",
            "input": {
              "type": "Struct",
              "name": "PriceOracle_admin_badge_address_Input",
              "fields": {
                "type": "Named",
                "named": []
              }
            },
            "output": {
              "type": "Custom",
              "type_id": 182,
              "generics": []
            },
            "export_name": "PriceOracle_admin_badge_address"
          }
        ]
      }
    }
  "#
  }
  
import! {
r#"
{
  "package_address": "package_sim1qyna563fqj6pqvpss763w9hw4vrdxgapfucx8esqyq0s5auvae",
  "blueprint_name": "SyntheticPool",
  "abi": {
    "structure": {
      "type": "Struct",
      "name": "SyntheticPool",
      "fields": {
        "type": "Named",
        "named": [
          [
            "oracle_address",
            {
              "type": "Custom",
              "type_id": 129,
              "generics": []
            }
          ],
          [
            "collateralization_threshold",
            {
              "type": "Custom",
              "type_id": 161,
              "generics": []
            }
          ],
          [
            "snx_resource_address",
            {
              "type": "Custom",
              "type_id": 182,
              "generics": []
            }
          ],
          [
            "usd_resource_address",
            {
              "type": "Custom",
              "type_id": 182,
              "generics": []
            }
          ],
          [
            "users",
            {
              "type": "Custom",
              "type_id": 131,
              "generics": [
                {
                  "type": "Custom",
                  "type_id": 182,
                  "generics": []
                },
                {
                  "type": "Struct",
                  "name": "User",
                  "fields": {
                    "type": "Named",
                    "named": [
                      [
                        "snx",
                        {
                          "type": "Custom",
                          "type_id": 179,
                          "generics": []
                        }
                      ],
                      [
                        "global_debt_share",
                        {
                          "type": "Custom",
                          "type_id": 179,
                          "generics": []
                        }
                      ]
                    ]
                  }
                }
              ]
            }
          ],
          [
            "synthetics",
            {
              "type": "HashMap",
              "key": {
                "type": "String"
              },
              "value": {
                "type": "Struct",
                "name": "SyntheticToken",
                "fields": {
                  "type": "Named",
                  "named": [
                    [
                      "asset_symbol",
                      {
                        "type": "String"
                      }
                    ],
                    [
                      "asset_address",
                      {
                        "type": "Custom",
                        "type_id": 182,
                        "generics": []
                      }
                    ],
                    [
                      "token_resource_address",
                      {
                        "type": "Custom",
                        "type_id": 182,
                        "generics": []
                      }
                    ]
                  ]
                }
              }
            }
          ],
          [
            "synthetics_mint_badge",
            {
              "type": "Custom",
              "type_id": 179,
              "generics": []
            }
          ],
          [
            "synthetics_global_debt_share_resource_address",
            {
              "type": "Custom",
              "type_id": 182,
              "generics": []
            }
          ]
        ]
      }
    },
    "fns": [
      {
        "ident": "instantiate_pool",
        "mutability": null,
        "input": {
          "type": "Struct",
          "name": "SyntheticPool_instantiate_pool_Input",
          "fields": {
            "type": "Named",
            "named": [
              [
                "arg0",
                {
                  "type": "Custom",
                  "type_id": 129,
                  "generics": []
                }
              ],
              [
                "arg1",
                {
                  "type": "Custom",
                  "type_id": 182,
                  "generics": []
                }
              ],
              [
                "arg2",
                {
                  "type": "Custom",
                  "type_id": 182,
                  "generics": []
                }
              ],
              [
                "arg3",
                {
                  "type": "Custom",
                  "type_id": 161,
                  "generics": []
                }
              ]
            ]
          }
        },
        "output": {
          "type": "Custom",
          "type_id": 129,
          "generics": []
        },
        "export_name": "SyntheticPool_instantiate_pool"
      },
      {
        "ident": "add_synthetic_token",
        "mutability": "Mutable",
        "input": {
          "type": "Struct",
          "name": "SyntheticPool_add_synthetic_token_Input",
          "fields": {
            "type": "Named",
            "named": [
              [
                "arg0",
                {
                  "type": "String"
                }
              ],
              [
                "arg1",
                {
                  "type": "Custom",
                  "type_id": 182,
                  "generics": []
                }
              ]
            ]
          }
        },
        "output": {
          "type": "Custom",
          "type_id": 182,
          "generics": []
        },
        "export_name": "SyntheticPool_add_synthetic_token"
      },
      {
        "ident": "stake",
        "mutability": "Mutable",
        "input": {
          "type": "Struct",
          "name": "SyntheticPool_stake_Input",
          "fields": {
            "type": "Named",
            "named": [
              [
                "arg0",
                {
                  "type": "Custom",
                  "type_id": 178,
                  "generics": []
                }
              ],
              [
                "arg1",
                {
                  "type": "Custom",
                  "type_id": 177,
                  "generics": []
                }
              ]
            ]
          }
        },
        "output": {
          "type": "Unit"
        },
        "export_name": "SyntheticPool_stake"
      },
      {
        "ident": "unstake",
        "mutability": "Mutable",
        "input": {
          "type": "Struct",
          "name": "SyntheticPool_unstake_Input",
          "fields": {
            "type": "Named",
            "named": [
              [
                "arg0",
                {
                  "type": "Custom",
                  "type_id": 178,
                  "generics": []
                }
              ],
              [
                "arg1",
                {
                  "type": "Custom",
                  "type_id": 161,
                  "generics": []
                }
              ]
            ]
          }
        },
        "output": {
          "type": "Custom",
          "type_id": 177,
          "generics": []
        },
        "export_name": "SyntheticPool_unstake"
      },
      {
        "ident": "mint",
        "mutability": "Mutable",
        "input": {
          "type": "Struct",
          "name": "SyntheticPool_mint_Input",
          "fields": {
            "type": "Named",
            "named": [
              [
                "arg0",
                {
                  "type": "Custom",
                  "type_id": 178,
                  "generics": []
                }
              ],
              [
                "arg1",
                {
                  "type": "Custom",
                  "type_id": 161,
                  "generics": []
                }
              ],
              [
                "arg2",
                {
                  "type": "String"
                }
              ]
            ]
          }
        },
        "output": {
          "type": "Custom",
          "type_id": 177,
          "generics": []
        },
        "export_name": "SyntheticPool_mint"
      },
      {
        "ident": "burn",
        "mutability": "Mutable",
        "input": {
          "type": "Struct",
          "name": "SyntheticPool_burn_Input",
          "fields": {
            "type": "Named",
            "named": [
              [
                "arg0",
                {
                  "type": "Custom",
                  "type_id": 178,
                  "generics": []
                }
              ],
              [
                "arg1",
                {
                  "type": "Custom",
                  "type_id": 177,
                  "generics": []
                }
              ]
            ]
          }
        },
        "output": {
          "type": "Unit"
        },
        "export_name": "SyntheticPool_burn"
      },
      {
        "ident": "get_total_global_debt",
        "mutability": "Immutable",
        "input": {
          "type": "Struct",
          "name": "SyntheticPool_get_total_global_debt_Input",
          "fields": {
            "type": "Named",
            "named": []
          }
        },
        "output": {
          "type": "Custom",
          "type_id": 161,
          "generics": []
        },
        "export_name": "SyntheticPool_get_total_global_debt"
      },
      {
        "ident": "get_snx_price",
        "mutability": "Immutable",
        "input": {
          "type": "Struct",
          "name": "SyntheticPool_get_snx_price_Input",
          "fields": {
            "type": "Named",
            "named": []
          }
        },
        "output": {
          "type": "Custom",
          "type_id": 161,
          "generics": []
        },
        "export_name": "SyntheticPool_get_snx_price"
      },
      {
        "ident": "get_asset_price",
        "mutability": "Immutable",
        "input": {
          "type": "Struct",
          "name": "SyntheticPool_get_asset_price_Input",
          "fields": {
            "type": "Named",
            "named": [
              [
                "arg0",
                {
                  "type": "Custom",
                  "type_id": 182,
                  "generics": []
                }
              ]
            ]
          }
        },
        "output": {
          "type": "Custom",
          "type_id": 161,
          "generics": []
        },
        "export_name": "SyntheticPool_get_asset_price"
      },
      {
        "ident": "get_user_summary",
        "mutability": "Mutable",
        "input": {
          "type": "Struct",
          "name": "SyntheticPool_get_user_summary_Input",
          "fields": {
            "type": "Named",
            "named": [
              [
                "arg0",
                {
                  "type": "Custom",
                  "type_id": 182,
                  "generics": []
                }
              ]
            ]
          }
        },
        "output": {
          "type": "String"
        },
        "export_name": "SyntheticPool_get_user_summary"
      },
      {
        "ident": "new_user",
        "mutability": "Immutable",
        "input": {
          "type": "Struct",
          "name": "SyntheticPool_new_user_Input",
          "fields": {
            "type": "Named",
            "named": []
          }
        },
        "output": {
          "type": "Custom",
          "type_id": 177,
          "generics": []
        },
        "export_name": "SyntheticPool_new_user"
      }
    ]
  }
}
"#
}

import! {
r#"
{
  "package_address": "package_sim1q8e4p0ut06dxeczmmhjlcs9mlkkhhml9ce58dlwxtugsfxedpm",
  "blueprint_name": "Radiswap",
  "abi": {
    "structure": {
      "type": "Struct",
      "name": "Radiswap",
      "fields": {
        "type": "Named",
        "named": [
          [
            "lp_resource_address",
            {
              "type": "Custom",
              "type_id": 182,
              "generics": []
            }
          ],
          [
            "lp_mint_badge",
            {
              "type": "Custom",
              "type_id": 179,
              "generics": []
            }
          ],
          [
            "a_pool",
            {
              "type": "Custom",
              "type_id": 179,
              "generics": []
            }
          ],
          [
            "b_pool",
            {
              "type": "Custom",
              "type_id": 179,
              "generics": []
            }
          ],
          [
            "fee",
            {
              "type": "Custom",
              "type_id": 161,
              "generics": []
            }
          ],
          [
            "lp_per_asset_ratio",
            {
              "type": "Custom",
              "type_id": 161,
              "generics": []
            }
          ]
        ]
      }
    },
    "fns": [
      {
        "ident": "instantiate_pool",
        "mutability": null,
        "input": {
          "type": "Struct",
          "name": "Radiswap_instantiate_pool_Input",
          "fields": {
            "type": "Named",
            "named": [
              [
                "arg0",
                {
                  "type": "Custom",
                  "type_id": 177,
                  "generics": []
                }
              ],
              [
                "arg1",
                {
                  "type": "Custom",
                  "type_id": 177,
                  "generics": []
                }
              ],
              [
                "arg2",
                {
                  "type": "Custom",
                  "type_id": 161,
                  "generics": []
                }
              ],
              [
                "arg3",
                {
                  "type": "String"
                }
              ],
              [
                "arg4",
                {
                  "type": "String"
                }
              ],
              [
                "arg5",
                {
                  "type": "String"
                }
              ],
              [
                "arg6",
                {
                  "type": "Custom",
                  "type_id": 161,
                  "generics": []
                }
              ]
            ]
          }
        },
        "output": {
          "type": "Tuple",
          "elements": [
            {
              "type": "Custom",
              "type_id": 129,
              "generics": []
            },
            {
              "type": "Custom",
              "type_id": 177,
              "generics": []
            }
          ]
        },
        "export_name": "Radiswap_instantiate_pool"
      },
      {
        "ident": "add_liquidity",
        "mutability": "Mutable",
        "input": {
          "type": "Struct",
          "name": "Radiswap_add_liquidity_Input",
          "fields": {
            "type": "Named",
            "named": [
              [
                "arg0",
                {
                  "type": "Custom",
                  "type_id": 177,
                  "generics": []
                }
              ],
              [
                "arg1",
                {
                  "type": "Custom",
                  "type_id": 177,
                  "generics": []
                }
              ]
            ]
          }
        },
        "output": {
          "type": "Tuple",
          "elements": [
            {
              "type": "Custom",
              "type_id": 177,
              "generics": []
            },
            {
              "type": "Custom",
              "type_id": 177,
              "generics": []
            }
          ]
        },
        "export_name": "Radiswap_add_liquidity"
      },
      {
        "ident": "remove_liquidity",
        "mutability": "Mutable",
        "input": {
          "type": "Struct",
          "name": "Radiswap_remove_liquidity_Input",
          "fields": {
            "type": "Named",
            "named": [
              [
                "arg0",
                {
                  "type": "Custom",
                  "type_id": 177,
                  "generics": []
                }
              ]
            ]
          }
        },
        "output": {
          "type": "Tuple",
          "elements": [
            {
              "type": "Custom",
              "type_id": 177,
              "generics": []
            },
            {
              "type": "Custom",
              "type_id": 177,
              "generics": []
            }
          ]
        },
        "export_name": "Radiswap_remove_liquidity"
      },
      {
        "ident": "swap",
        "mutability": "Mutable",
        "input": {
          "type": "Struct",
          "name": "Radiswap_swap_Input",
          "fields": {
            "type": "Named",
            "named": [
              [
                "arg0",
                {
                  "type": "Custom",
                  "type_id": 177,
                  "generics": []
                }
              ]
            ]
          }
        },
        "output": {
          "type": "Custom",
          "type_id": 177,
          "generics": []
        },
        "export_name": "Radiswap_swap"
      },
      {
        "ident": "get_pair",
        "mutability": "Immutable",
        "input": {
          "type": "Struct",
          "name": "Radiswap_get_pair_Input",
          "fields": {
            "type": "Named",
            "named": []
          }
        },
        "output": {
          "type": "Tuple",
          "elements": [
            {
              "type": "Custom",
              "type_id": 182,
              "generics": []
            },
            {
              "type": "Custom",
              "type_id": 182,
              "generics": []
            }
          ]
        },
        "export_name": "Radiswap_get_pair"
      }
    ]
  }
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
