use scrypto::prelude::*;
use std::cmp::Ordering;

// =======================
// Blueprint + Core Logic
// =======================

blueprint! {
    /// This struct defines the fields which are required to implement a store which has tiers that control how much can
    /// be withdrawn from the store depending on the badges which have been presented.
    ///
    /// # Example:
    ///
    /// Say that we wish to create a token store which allows us to have a withdraw schedule which looks a little
    /// something like the following:
    ///
    /// * When a founder's badge + 2 admin badges are present: authorize the withdraw of up to 200 XRD
    /// * When a founder's badge + 4 admin badges are present: authorize the withdraw of up to 1,000 XRD
    /// * When a founder's badge + 8 admin badges are present: authorize the withdraw of up to 10,000 XRD
    /// * When a founder's badge + 10 admin badges are present: authorize the withdraw all locked up XRD
    ///
    /// This is why this store is said to be tiered as it includes multiple tiers for withdrawals.
    struct TieredFund {
        /// A hashmap which maps a given access rule to the withdraw limit associated with the access rule.
        withdraw_limits: HashMap<AccessRule, WithdrawLimit>,

        /// A hashmap which is used to keep track of the amount of tokens that have been withdrawn through each access
        /// rule. Keep in mind that the amount here could be reset to allow for more withdrawals to take place. So, this
        /// should not be looked at as the total amount withdrawn
        withdraw_history: HashMap<AccessRule, Decimal>,

        /// The vault containing the tokens for which the tiering system will work. Keep in mind that this is a single
        /// vault and not a vector of vaults. This means that a single component can only provide tiering for a single
        /// resource. If you wish to have such functionality with multiple tokens then you should instantiate multiple
        /// components.
        vault: Vault,
    }

    impl TieredFund {
        /// Instantiates a new TieredFund component.
        ///
        /// This function allows for the creation of a new tiered fund account along with a badge which is given admin
        /// rights. This function returns the address of the newly instantiated component alongside the a bucket of the
        /// badge containing the admin badge. The admin badge returned may be used for:
        /// * The addition of new tiers through `add_tier`.
        /// * The removal of tiers through `remove_tier`.
        /// * Resetting the tier history through `reset_tier_history`.
        /// * The modification of tier limits through `update_tier_limit`.
        ///
        /// # Arguments:
        ///
        /// * `tokens_resource_address` (ResourceAddress) - The resource address of the tokens that will be used for the
        /// fund.
        ///
        /// # Returns:
        ///
        /// * `ComponentAddress` - The component address of the newly instantiated tiered fund.
        /// * `Bucket` - A bucket containing the admin badge.
        pub fn instantiate_simple_tiered_fund(
            tokens_resource_address: ResourceAddress,
        ) -> (ComponentAddress, Bucket) {
            // Creating the admin badge which we would like to give the authority to reset tiers or add additional tiers
            let admin_badge: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Admin Badge")
                .metadata("description", "An admin badge which comes as part of the tiered fund and is used for admin operations")
                .metadata("symbol", "ADMIN")
                .initial_supply(1);

            return (
                Self::instantiate_custom_tiered_fund(
                    rule!(require(admin_badge.resource_address())),
                    tokens_resource_address,
                ),
                admin_badge,
            );
        }

        /// Instantiates a new TieredFund component with a custom access rule.
        ///
        /// This function allows the caller to instantiate a new tiered fund component with a custom user-defined access
        /// rule which governs the authorization to the methods responsible for the administrative operations on the
        /// fund.
        ///
        /// This function performs a number of checks before creating a new tiered fund component:
        ///
        /// * **Check 1:** Checks to ensure that the token used for fund is a fungible token.
        ///
        /// # Arguments:
        ///
        /// * `administration_rule` (AccessRule) - An access rule which will be used for the administrative methods on
        /// the component. This rule will be used for:
        ///     * The addition of new tiers through `add_tier`.
        ///     * The removal of tiers through `remove_tier`.
        ///     * Resetting the tier history through `reset_tier_history`.
        ///     * The modification of tier limits through `update_tier_limit`.
        /// * `tokens_resource_address` (ResourceAddress) - The resource address of the tokens that will be used for the
        /// fund.
        ///
        /// # Returns:
        ///
        /// * `ComponentAddress` - The component address of the newly instantiated tiered fund.
        pub fn instantiate_custom_tiered_fund(
            administration_rule: AccessRule,
            tokens_resource_address: ResourceAddress,
        ) -> ComponentAddress {
            // Performing the checks to determine if the tiered fund can be created
            assert!(
                !matches!(borrow_resource_manager!(tokens_resource_address).resource_type(), ResourceType::NonFungible),
                "[Custom Instantiation]: A tiered fund can not be created with a non-fungible token."
            );

            // At this point we know that the creation of the component can go through.

            // Defining the access rules for the component.
            let access_rules: AccessRules = AccessRules::new()
                .method("add_tier", administration_rule.clone())
                .method("remove_tier", administration_rule.clone())
                .method("reset_tier_history", administration_rule.clone())
                .method("update_tier_limit", administration_rule.clone())
                .default(rule!(allow_all));

            // Instantiating the component and returning its address
            return Self {
                withdraw_limits: HashMap::new(),
                withdraw_history: HashMap::new(),
                vault: Vault::new(tokens_resource_address),
            }
            .instantiate()
            .add_access_check(access_rules)
            .globalize();
        }

        /// Adds a new tier to the fund.
        ///
        /// This method performs a number of checks before adding a new tier:
        ///
        /// * **Check 1:** Checks that there does not already exist a tier with the same AccessRule to it.
        ///
        /// # Arguments:
        ///
        /// * `access_rule` (AccessRule) - The access rule which the withdrawal limit will be attached to.
        /// * `withdraw_limit` (AccessRule) - An that which defines the limit on withdrawal which will be imposed on the
        /// access_rule.
        pub fn add_tier(&mut self, access_rule: AccessRule, withdraw_limit: WithdrawLimit) {
            // Checking the tier can be added
            assert!(
                !self.withdraw_history.contains_key(&access_rule),
                "[Add Tier]: A tier with the same access rule already exists."
            );

            // At this point we know that the tier can be added to our records.
            self.withdraw_limits
                .insert(access_rule.clone(), withdraw_limit);
            self.withdraw_history.insert(access_rule, dec!("0"));
        }

        /// Removes a specific tier from the fund.
        ///
        /// This method performs a number of checks before removing the tier:
        ///
        /// * **Check 1:** Checks the tier does already exist before removing it.
        ///
        /// # Arguments:
        ///
        /// * `access_rule` (AccessRule) - The access rule to remove from the fund.
        pub fn remove_tier(&mut self, access_rule: AccessRule) {
            // Checking the tier can be removed
            assert!(
                self.withdraw_history.contains_key(&access_rule),
                "[Remove Tier]: Can't remove a tier which does not already exist."
            );

            // At this point we know that the tier can be removed
            self.withdraw_limits.remove(&access_rule);
            self.withdraw_history.remove(&access_rule);
        }

        /// Rests the history for a specific tier.
        ///
        /// This method performs a number of checks before removing the tier:
        ///
        /// * **Check 1:** Checks the tier does already exist before removing it.
        ///
        /// # Arguments:
        ///
        /// * `access_rule` (AccessRule) - The access rule to reset the history for.
        pub fn reset_tier_history(&mut self, access_rule: AccessRule) {
            // Checking the tier's history can be reset
            assert!(
                self.withdraw_history.contains_key(&access_rule),
                "[Reset Tier History]: Can't reset history for a tier which does not already exist."
            );

            // At this point we know that the tier's history can be reset
            *self.withdraw_history.get_mut(&access_rule).unwrap() = Decimal::zero();
        }

        /// Updates the withdrawal limit for a given tier.
        ///
        /// This method performs a number of checks before removing the tier:
        ///
        /// * **Check 1:** Checks the tier does already exist before removing it.
        ///
        /// # Arguments:
        ///
        /// * `access_rule` (AccessRule) - The access rule to reset the history for.
        pub fn update_tier_limit(
            &mut self,
            access_rule: AccessRule,
            withdraw_limit: WithdrawLimit,
        ) {
            // Checking the tier's limit can be updated
            assert!(
                self.withdraw_history.contains_key(&access_rule),
                "[Update Tier Limit]: Can't update tier limit for a tier which does not already exist."
            );

            // At this point we know that the tier's limit can be updated.
            *self.withdraw_limits.get_mut(&access_rule).unwrap() = withdraw_limit;
        }

        /// Withdraws funds from the component.
        ///
        /// This method is used to withdraw funds from the component with the limit of the amount that can be withdrawn
        /// varying depending on the access rules which the vector of provided proofs satisfies. In the case that the 
        /// vector of proofs satisfies multiple `AccessRules` then the withdrawal limit will be set to the maximum of 
        /// the rules.
        /// 
        /// This method performs two main checks before the withdraw of the tokens happen:
        /// 
        /// * **Check 1:** Checks that the provided proofs satisfy at least one access rule.
        /// * **Check 2:** Checks that the satisfied rule's limit has not been reached (if exists).
        /// 
        /// # Arguments:
        /// 
        /// * `withdraw_amount` (Decimal) - A decimal which defines the amount of tokens to withdraw.
        /// * `proof` (Vec<Proof>) - A vector of proofs of the badges used for the authorization checks of the fund 
        /// withdrawal
        /// 
        /// # Returns:
        /// 
        /// * `Bucket` - A bucket of the withdrawn tokens.
        pub fn withdraw(&mut self, withdraw_amount: Decimal, proofs: Vec<Proof>) -> Bucket {
            // Getting the limits which are valid for the proofs that we currently have
            let valid_limits: HashMap<AccessRule, WithdrawLimit> = self
                .withdraw_limits
                .iter()
                .filter(|(access_rule, _)| access_rule.check(&proofs[..]))
                .map(|(x, y)| (x.clone(), y.clone()))
                .collect::<HashMap<AccessRule, WithdrawLimit>>();

            // Getting the maximum pair of access rule and withdraw limit according to the withdraw limit
            let limit: Option<(AccessRule, WithdrawLimit)> = valid_limits
                .iter()
                .max_by(|a, b| a.1.cmp(&b.1))
                .map(|(x, y)| (x.clone(), y.clone()));

            match limit {
                Some((access_rule, withdraw_limit)) => {
                    // Ensuring that the amount that they wish to withdraw would still be within their limits
                    match withdraw_limit {
                        WithdrawLimit::Finite(finite_withdraw_limit) => {
                            assert!(
                                withdraw_amount
                                    + self.withdraw_history.get(&access_rule).unwrap().clone()
                                    <= finite_withdraw_limit,
                                "[Withdraw]: Can not withdraw more you are allowed to"
                            );
                        }
                        _ => {}
                    };

                    // If we get to this point, then we are in the clear! The withdrawal of the funds can take place
                    // without any issues as we're sure that the checks have all passed successfully.

                    // Adding the withdrawal amount to the history. The history of all of the valid access rules gets
                    // updated to reflect on the amount that they've withdrawn so far.
                    let keys: Vec<AccessRule> = valid_limits.keys().cloned().collect();
                    for key in keys.iter() {
                        *self.withdraw_history.get_mut(key).unwrap() += withdraw_amount;
                    }

                    // Withdrawing the funds and returning them
                    return self.vault.take(withdraw_amount);
                }
                None => {
                    assert!(
                        false,
                        "[Withdraw]: Provided proofs do not satisfy any of the tier AccessRules."
                    );
                    panic!(""); // Empty panic kept there to allow for uneven match statement arms.
                }
            }
        }
    
        /// Deposits tokens into the fund
        /// 
        /// # Arguments:
        /// 
        /// * `tokens` (Bucket) - A bucket of tokens to deposit into the fund.
        pub fn deposit(&mut self, tokens: Bucket) {
            // Checking if the deposit can be made
            assert_eq!(tokens.resource_address(), self.vault.resource_address(), "[Deposit]: Incorrect type of token");

            // Depositing the tokens
            self.vault.put(tokens);
        }
    }
}

// =================
// Additional Types
// =================

/// An enum which defines the amount of funds that can be withdrawn, typically in relation to some access rule. The
/// limit can be finite or infinite.
#[derive(Describe, Encode, Decode, TypeId, Debug, Clone)]
pub enum WithdrawLimit {
    /// A variant which defines a finite withdrawal limit with a given amount of tokens that can be withdrawn.
    Finite(Decimal),

    /// A variant which defines that an infinite amount may be withdrawn.
    Infinite,
}

impl Eq for WithdrawLimit {}

impl PartialEq for WithdrawLimit {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (WithdrawLimit::Finite(a), WithdrawLimit::Finite(b)) => a == b,
            (WithdrawLimit::Infinite, WithdrawLimit::Infinite) => true,
            _ => false,
        }
    }
}

impl Ord for WithdrawLimit {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (WithdrawLimit::Finite(a), WithdrawLimit::Finite(b)) => a.cmp(b),
            (WithdrawLimit::Infinite, WithdrawLimit::Infinite) => Ordering::Equal,
            (WithdrawLimit::Infinite, _) => Ordering::Greater,
            (_, WithdrawLimit::Infinite) => Ordering::Less,
        }
    }
}

impl PartialOrd for WithdrawLimit {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
