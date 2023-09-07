use scrypto::prelude::*;

#[blueprint]
mod regulated_token {
    enable_method_auth! {
        roles {
            freeze_admin => updatable_by: [];
            general_admin => updatable_by: [];
        },
        methods {
            toggle_transfer_freeze => restrict_to: [freeze_admin];
            collect_payments => restrict_to: [general_admin];
            advance_stage => restrict_to: [general_admin];
            get_current_stage => PUBLIC;
            buy_token => PUBLIC;
        }
    }
    struct RegulatedToken {
        token_supply: Vault,
        collected_xrd: Vault,
        current_stage: u8,
        admin_badge_address: ResourceAddress,
        freeze_admin_badge_address: ResourceAddress,
    }

    impl RegulatedToken {
        pub fn instantiate_regulated_token(
        ) -> (Global<RegulatedToken>, FungibleBucket, FungibleBucket) {
            // We are allocating a ComponentAddress used for our actor virtual badge and provide
            // minting & transfer authority to our component.
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(RegulatedToken::blueprint_id());

            // Creating two resources we will use as badges and return to our instantiator
            let general_admin = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata! (
                    init {
                        "name" => "RegulatedToken general admin badge".to_string(), locked;
                    }
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                ))
                .mint_initial_supply(1);

            let freeze_admin = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata! (
                    init {
                        "name" => "RegulatedToken freeze-only badge".to_string(), locked;
                    }
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                ))
                .mint_initial_supply(1);

            // Next we will create our regulated token with an initial fixed supply of 100 and the appropriate permissions
            let access_rule: AccessRule = rule!(
                require(general_admin.resource_address())
                    || require(global_caller(component_address))
            );
            let regulated_tokens = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! (
                    roles {
                        metadata_setter => access_rule.clone();
                        metadata_setter_updater => access_rule.clone();
                        metadata_locker => access_rule.clone();
                        metadata_locker_updater => access_rule.clone();
                    },
                    init {
                        "name" => "Regulo".to_string(), locked;
                        "symbol" => "REG".to_string(), locked;
                        "stage" => "Stage 1 - Fixed supply, may be restricted transfer".to_string(), updatable;
                    }
                ))
                .freeze_roles(freeze_roles!(
                    freezer => rule!(require(freeze_admin.resource_address()));
                    freezer_updater => access_rule.clone();
                ))
                .withdraw_roles(withdraw_roles!(
                    withdrawer => rule!(require(freeze_admin.resource_address()));
                    withdrawer_updater => access_rule.clone();
                ))
                .recall_roles(recall_roles!(
                    recaller => access_rule.clone();
                    recaller_updater => access_rule.clone();
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(deny_all);
                    minter_updater => access_rule.clone();
                ))
                .mint_initial_supply(100);

            let component = Self {
                token_supply: Vault::with_bucket(regulated_tokens.into()),
                collected_xrd: Vault::new(XRD),
                current_stage: 1,
                admin_badge_address: general_admin.resource_address(),
                freeze_admin_badge_address: freeze_admin.resource_address(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles!(
                freeze_admin => rule!(require(freeze_admin.resource_address()));
                general_admin => rule!(require(general_admin.resource_address()));
            ))
            .with_address(address_reservation)
            .globalize();

            (component, general_admin, freeze_admin)
        }

        /// The freeze admin badge may be used to freeze or unfreeze consumer transfers of the supply
        pub fn toggle_transfer_freeze(&self, set_frozen: bool) {
            // Note that this operation will fail if the token has reached stage 3 and the token behavior has been locked
            let token_resource_manager = self.token_supply.resource_manager();

            if set_frozen {
                token_resource_manager
                    .set_withdrawable(rule!(require(self.freeze_admin_badge_address)));
                info!("Token transfer is now RESTRICTED");
            } else {
                token_resource_manager.set_withdrawable(rule!(allow_all));
                info!("Token is now freely transferrable");
            }
        }

        pub fn get_current_stage(&self) -> u8 {
            info!("Current stage is {}", self.current_stage);
            self.current_stage
        }

        /// Permit the proper authority to withdraw our collected XRD
        pub fn collect_payments(&mut self) -> Bucket {
            self.collected_xrd.take_all()
        }

        pub fn advance_stage(&mut self) {
            assert!(self.current_stage <= 2, "Already at final stage");
            let token_resource_manager = self.token_supply.resource_manager();

            if self.current_stage == 1 {
                // Advance to stage 2
                // Token will still be restricted transfer upon admin demand, but we will mint beyond the initial supply as required
                self.current_stage = 2;

                // Update token's metadata to reflect the current stage
                token_resource_manager.set_metadata(
                    "stage",
                    "Stage 2 - Unlimited supply, may be restricted transfer".to_string(),
                );

                // Enable minting for the token
                token_resource_manager.set_mintable(rule!(
                    require(self.admin_badge_address)
                        || require(global_caller(Runtime::global_address()))
                ));
                info!("Advanced to stage 2");
            } else {
                // Advance to stage 3
                // Token will no longer be regulated
                // Restricted transfer will be permanently turned off, supply will be made permanently immutable
                self.current_stage = 3;

                // Update token's metadata to reflect the final stage
                token_resource_manager.set_metadata(
                    "stage",
                    "Stage 3 - Unregulated token, fixed supply".to_string(),
                );

                // Set our behavior appropriately now that the regulated period has ended
                token_resource_manager.set_mintable(rule!(deny_all));
                token_resource_manager.set_freezeable(rule!(deny_all));
                token_resource_manager.set_recallable(rule!(deny_all));
                token_resource_manager.set_withdrawable(rule!(allow_all));
                token_resource_manager.set_metadata_role("metadata_setter", rule!(deny_all));
                token_resource_manager
                    .set_metadata_role("metadata_setter_updater", rule!(deny_all));

                // Permanently prevent the behavior of the token from changing
                token_resource_manager.lock_mintable();
                token_resource_manager.lock_withdrawable();
                token_resource_manager.lock_freezeable();
                token_resource_manager.lock_recallable();
                token_resource_manager.lock_updatable_metadata();

                // With the resource behavior forever locked, our internal authority badge no longer has any use
                // We will burn our internal badge, and the holders of the other badges may burn them at will
                // Our badge has the allows everybody to burn, so there's no need to provide a burning authority

                info!("Advanced to stage 3");
            }
        }

        /// Buy a quantity of tokens, if the supply on-hand is sufficient, or if current rules permit minting additional supply.
        /// The system will *always* allow buyers to purchase available tokens, even when the token transfers are otherwise frozen
        pub fn buy_token(&mut self, quantity: Decimal, mut payment: Bucket) -> (Bucket, Bucket) {
            assert!(
                quantity > dec!("0"),
                "Can't sell you nothing or less than nothing"
            );

            // Early birds who buy during stage 1 get a discounted rate
            let price: Decimal = if self.current_stage == 1 {
                dec!("50")
            } else {
                dec!("100")
            };

            // Take what we're owed
            self.collected_xrd
                .put(payment.take(price.checked_mul(quantity).unwrap()));

            // Can we fill the desired quantity from current supply?
            let extra_demand = quantity.checked_sub(self.token_supply.amount()).unwrap();
            if extra_demand <= dec!("0") {
                // Take the required quantity, and return it along with any change
                // The token may currently be under restricted transfer, so we will authorize our withdrawal
                let tokens = self.token_supply.take(quantity);

                return (tokens, payment);
            } else {
                // We will attempt to mint the shortfall
                // If we are in stage 1 or 3, this action will fail, and it would probably be a good idea to tell the user this
                // For the purposes of example, we will blindly attempt to mint
                let mut tokens = self.token_supply.resource_manager().mint(extra_demand);

                // Combine the new tokens with whatever was left in supply to meet the full quantity
                let existing_tokens = self.token_supply.take_all();
                tokens.put(existing_tokens);

                // Return the tokens, along with any change
                return (tokens, payment);
            }
        }
    }
}
