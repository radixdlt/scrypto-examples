use scrypto::prelude::*;
use std::cmp::Ordering;

// =======================
// Blueprint + Core Logic
// =======================

blueprint! {
    /// This struct defines the fields of a store of funds which can be controlled by one or more entities which allows 
    /// other entities (called withdraw authorities in this context) to withdraw funds from the component within their
    /// allowable limits which may be finite or infinite. 
    /// 
    /// In this blueprint, entities which are given the right to withdraw are called authorized entities and not any 
    /// other name. This is because this blueprint allows for entities with badges that satisfy `AccessRule`s to 
    /// withdraw from the blueprint. Authorized entities may be as simple as a single badge or as complex as an 
    /// organizational structure where multiple badges need to be involved for people on different levels of the 
    /// organization. 
    /// 
    /// An example of where this blueprint can be useful is the following: say that we wish to create a token store 
    /// which allows us to have a withdraw authority schedule:
    ///
    /// * When an admin badge or supervisor badge are present: authorize the withdraw of up to 100 XRD
    /// * When a founder's badge + 2 admin badges are present: authorize the withdraw of up to 200 XRD
    /// * When a founder's badge + 4 admin badges are present: authorize the withdraw of up to 1,000 XRD
    /// * When a founder's badge + 8 admin badges are present: authorize the withdraw of up to 10,000 XRD
    /// * When a founder's badge + 10 admin badges are present: authorize the withdraw all locked up XRD
    ///
    /// While at first glace, it might appear that the problem of limited vault withdrawals can be solved by creating
    /// multiple non-fungible badges and passing them to the involved entities, this solution makes the assumption that
    /// the entities for which we want to give withdraw authority are all singular non-complex entities which is a wrong
    /// assumption in many cases. 
    /// 
    /// This blueprint can be used to model the above mentioned behavior using the `AccessRule`s functionality provided 
    /// in Scrypto v0.4.0.
    /// 
    /// This blueprint makes no assumptions about the auth structure of the instantiator of the component. Meaning, this
    /// blueprint has constructors which allow for the setting of custom administration rule to to fit the needs of all
    /// kinds of people who might require to use this blueprint.
    struct LimitedWithdrawVault {
        /// A hashmap which maps a given access rule to the withdraw limit associated with the access rule. Only when 
        /// the access rule is satisfied, does this withdraw limit kick into place and begin to matter.
        withdraw_limits: HashMap<AccessRule, WithdrawLimit>,

        /// A hashmap which is used to keep track of the amount of tokens that have been withdrawn through each access
        /// rule. Keep in mind that the amount here could be reset to allow for more withdrawals to take place. So, this
        /// should **not** be looked at as the total amount withdrawn. Instead, this is introduced as a way to stop an
        /// access rule from making multiple withdrawals. Meaning, this is used to ensure that the withdraw limit is a 
        /// global limit and not a per-call limit.
        withdraw_history: HashMap<AccessRule, Decimal>,

        /// The underlying vault which will be storing the funds of the component. Keep in mind that this is a single
        /// vault and not a vector of vaults. This means that a single component can only provide functionality for a 
        /// single resource. If you wish to have such functionality with multiple tokens then you should instantiate 
        /// multiple components.
        vault: Vault,
    }

    impl LimitedWithdrawVault {
        /// Instantiates a new LimitedWithdrawVault component.
        ///
        /// This function allows for the creation of a new limited withdraw vault account along with a badge which is 
        /// given admin rights. This function returns the address of the newly instantiated component alongside the a 
        /// bucket of the badge containing the admin badge. The admin badge returned may be used for:
        /// 
        /// * The addition of new withdraw authorities through `add_withdraw_authority`.
        /// * The removal of withdraw authorities through `remove_withdraw_authority`.
        /// * Resetting the authority history through `reset_withdraw_history_of_authority`.
        /// * The modification of authority limits through `update_withdraw_authority_limit`.
        ///
        /// # Arguments:
        ///
        /// * `tokens_resource_address` (ResourceAddress) - The resource address of the tokens that will be used for the
        /// fund.
        ///
        /// # Returns:
        ///
        /// * `ComponentAddress` - The component address of the newly instantiated limited withdraw vault.
        /// * `Bucket` - A bucket containing the admin badge.
        pub fn instantiate_simple_limited_withdraw_vault(
            tokens_resource_address: ResourceAddress,
        ) -> (ComponentAddress, Bucket) {
            // Creating the admin badge which we would like to give the authority to reset withdraw authorities' history
            // or add additional withdraw authorities
            let admin_badge: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Admin Badge")
                .metadata("description", "An admin badge which comes as part of the limited withdraw vault and is used for admin operations")
                .metadata("symbol", "ADMIN")
                .initial_supply(1);

            return (
                Self::instantiate_custom_limited_withdraw_vault(
                    rule!(require(admin_badge.resource_address())),
                    tokens_resource_address,
                ),
                admin_badge,
            );
        }

        /// Instantiates a new LimitedWithdrawVault component with a custom access rule.
        ///
        /// This function allows the caller to instantiate a limited withdraw vault which has a custom administration
        /// rule for the methods which require "admin" rights. Administrative rights are rights to methods on the 
        /// component which are typically seen as administrative such as the addition of new withdrawal authorities, the 
        /// removal of them, and the changing of withdraw limits. This is a non-exhaustive list of the actions which may
        /// be considered administrative. A more complete list of administrative methods can be seen below in the 
        /// arguments section.
        ///
        /// # Arguments:
        ///
        /// * `administration_rule` (AccessRule) - An access rule which will be used for the administrative methods on
        /// the component. This rule will be used for:
        ///     * The addition of new authorities through `add_withdraw_authority`.
        ///     * The removal of authorities through `remove_withdraw_authority`.
        ///     * Resetting the authority history through `reset_withdraw_history_of_authority`.
        ///     * The modification of authority limits through `update_withdraw_authority_limit`.
        /// * `tokens_resource_address` (ResourceAddress) - The resource address of the tokens that will be used for the
        /// vault.
        ///
        /// # Returns:
        ///
        /// * `ComponentAddress` - The component address of the newly instantiated limited withdraw vault.
        pub fn instantiate_custom_limited_withdraw_vault(
            administration_rule: AccessRule,
            tokens_resource_address: ResourceAddress,
        ) -> ComponentAddress {
            // Defining the access rules for the component.
            let access_rules: AccessRules = AccessRules::new()
                .method("withdraw", rule!(allow_all))
                .method("deposit", rule!(allow_all))
                .default(administration_rule);

            // Instantiating the component and returning its address
            return Self::instantiate_bare_bone_limited_withdraw_vault(access_rules, tokens_resource_address);
        }
        
        /// Instantiates a new LimitedWithdrawVault component with completely custom access rules.
        ///
        /// This function allows the caller to instantiate a limited withdraw vault which has a custom administration
        /// rule for the methods which require "admin" rights. Administrative rights are rights to methods on the 
        /// component which are typically seen as administrative such as the addition of new withdrawal authorities, the 
        /// removal of them, and the changing of withdraw limits. This is a non-exhaustive list of the actions which may
        /// be considered administrative. A more complete list of administrative methods can be seen below in the 
        /// arguments section.
        ///
        /// This function performs a number of checks before creating a new limited withdraw vault component:
        ///
        /// * **Check 1:** Checks to ensure that the token used for vault is a fungible token.
        ///
        /// # Arguments:
        ///
        /// * `access_rules` (AccessRules) - The access rules which the caller wishes to use for the methods on this 
        /// newly created component. The following are the methods which you can provide auth rules for:
        ///     * `add_withdraw_authority` - This is typically an administrative method.
        ///     * `remove_withdraw_authority` - This is typically an administrative method.
        ///     * `reset_withdraw_history_of_authority` - This is typically an administrative method.
        ///     * `update_withdraw_authority_limit` - This is typically an administrative method.
        ///     * `withdraw` - This is typically not an administrative method.
        ///     * `deposit` - This is typically not an administrative method.
        /// * `tokens_resource_address` (ResourceAddress) - The resource address of the tokens that will be used for the
        /// vault.
        ///
        /// # Returns:
        ///
        /// * `ComponentAddress` - The component address of the newly instantiated limited withdraw vault.
        /// 
        /// # Note:
        /// 
        /// This function provides you with a great deal of flexibility in terms of how you want to set your auth but
        /// also makes it possible to "shoot yourself in the leg" if you incorrectly setup the access rules for the 
        /// methods. Use this function with caution and only use it you understand exactly what you're doing. If all
        /// that you care about is creating a more complex limited withdraw vault, then consider using other functions
        /// like `instantiate_custom_limited_withdraw_vault`.
        pub fn instantiate_bare_bone_limited_withdraw_vault(
            access_rules: AccessRules,
            tokens_resource_address: ResourceAddress,
        ) -> ComponentAddress {
            // Performing the checks to determine if the limited withdraw vault can be created
            assert!(
                !matches!(borrow_resource_manager!(tokens_resource_address).resource_type(), ResourceType::NonFungible),
                "[Bare Bone Instantiation]: A limited withdraw vault can not be created with a non-fungible token."
            );

            // At this point we know that the creation of the component can go through.
           
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

        /// Adds a new withdraw authority to the the list of authorities.
        ///
        /// This method performs a single check before adding a new withdraw authority:
        ///
        /// * **Check 1:** Checks that there does not already exist a authority with the same AccessRule to it.
        ///
        /// # Arguments:
        ///
        /// * `access_rule` (AccessRule) - The access rule which the withdrawal limit will be attached to.
        /// * `withdraw_limit` (AccessRule) - A WithdrawLimit which defines the limit on withdrawal which will be 
        /// imposed on the access_rule.
        pub fn add_withdraw_authority(&mut self, access_rule: AccessRule, withdraw_limit: WithdrawLimit) {
            // Checking the authority can be added
            assert!(
                !self.withdraw_history.contains_key(&access_rule),
                "[Add Authority]: A authority with the same access rule already exists."
            );

            // At this point we know that the authority can be added to our records.
            self.withdraw_limits
                .insert(access_rule.clone(), withdraw_limit);
            self.withdraw_history.insert(access_rule, dec!("0"));
        }

        /// Removes a specific withdraw authority from the limited withdraw vault.
        ///
        /// This method performs a single check before removing the authority:
        ///
        /// * **Check 1:** Checks the authority does already exist before removing it.
        ///
        /// # Arguments:
        ///
        /// * `access_rule` (AccessRule) - The access rule to remove from the limited withdraw vault.
        pub fn remove_withdraw_authority(&mut self, access_rule: AccessRule) {
            // Checking the authority can be removed
            assert!(
                self.withdraw_history.contains_key(&access_rule),
                "[Remove Authority]: Can't remove a authority which does not already exist."
            );

            // At this point we know that the authority can be removed
            self.withdraw_limits.remove(&access_rule);
            self.withdraw_history.remove(&access_rule);
        }

        /// Rests the history for a specific withdraw authority.
        ///
        /// This method performs a single check before removing the authority:
        ///
        /// * **Check 1:** Checks the authority does already exist before removing it.
        ///
        /// # Arguments:
        ///
        /// * `access_rule` (AccessRule) - The access rule to reset the history for.
        pub fn reset_withdraw_history_of_authority(&mut self, access_rule: AccessRule) {
            // Checking the authority's history can be reset
            assert!(
                self.withdraw_history.contains_key(&access_rule),
                "[Reset Authority History]: Can't reset history for a authority which does not already exist."
            );

            // At this point we know that the authority's history can be reset
            *self.withdraw_history.get_mut(&access_rule).unwrap() = Decimal::zero();
            info!(
                "[Reset authority History]: Access Rule {:?} now has a withdraw history of: {}",
                access_rule,
                self.withdraw_history.get(&access_rule).unwrap()
            );
        }

        /// Updates the withdrawal limit for a given authority.
        ///
        /// This method performs a single check before removing the authority:
        ///
        /// * **Check 1:** Checks the authority does already exist before removing it.
        ///
        /// # Arguments:
        ///
        /// * `access_rule` (AccessRule) - The access rule to reset the history for.
        pub fn update_withdraw_authority_limit(
            &mut self,
            access_rule: AccessRule,
            withdraw_limit: WithdrawLimit,
        ) {
            // Checking the authority's limit can be updated
            assert!(
                self.withdraw_history.contains_key(&access_rule),
                "[Update Authority Limit]: Can't update authority limit for a authority which does not already exist."
            );

            // At this point we know that the authority's limit can be updated.
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
            info!(
                "[Withdraw]: Passed proofs satisfied the auth rules: {:?}",
                valid_limits
            );

            // Getting the maximum pair of access rule and withdraw limit according to the withdraw limit
            let limit: Option<(AccessRule, WithdrawLimit)> = valid_limits
                .iter()
                .max_by(|a, b| a.1.cmp(&b.1))
                .map(|(x, y)| (x.clone(), y.clone()));
            info!(
                "[Withdraw]: Maximum withdraw limit for the rule is: {:?}",
                limit
            );

            match limit {
                Some((access_rule, withdraw_limit)) => {
                    // Ensuring that the amount that they wish to withdraw would still be within their limits. The limit
                    // is not a per-call limit. It is a limit on how much, in general, can a caller(s) which satisfy
                    // this access rule withdraw from the component.
                    match withdraw_limit {
                        WithdrawLimit::Finite(finite_withdraw_limit) => {
                            let total_withdrawn: Decimal = withdraw_amount
                                + self.withdraw_history.get(&access_rule).unwrap().clone();
                            assert!(
                                total_withdrawn <= finite_withdraw_limit,
                                "[Withdraw]: Your total withdraw would be {} but you're only allowed to withdraw {}.",
                                total_withdrawn, finite_withdraw_limit
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
                        "[Withdraw]: Provided proofs do not satisfy any of the authority AccessRules."
                    );
                    panic!(""); // Empty panic kept here to allow for uneven match statement arms.
                }
            }
        }

        /// Deposits tokens into the vault
        ///
        /// # Arguments:
        ///
        /// * `tokens` (Bucket) - A bucket of tokens to deposit into the vault.
        pub fn deposit(&mut self, tokens: Bucket) {
            // Checking if the deposit can be made
            assert_eq!(
                tokens.resource_address(),
                self.vault.resource_address(),
                "[Deposit]: Can not deposit tokens as they do not belong to this vault."
            );

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
