use scrypto::prelude::*;

// -----------------
// Additional Types
// -----------------

/// An enum which defines the amount of funds that can be withdrawn, typically in association to some access rule. The
/// limit can be finite or infinite.
#[derive(Describe, Encode, Decode, TypeId)]
enum WithdrawLimit {
    /// A variant which defines a finite withdrawal limit with a given amount of tokens that can be withdrawn.
    Finite(Decimal),
    
    /// A variant which defines that an infinite amount may be withdrawn.
    Infinite
}

// -----------------------
// Blueprint + Core Logic
// -----------------------

blueprint! {
    /// This struct defines the fields which are required to implement a store which has tiers that control how much can
    /// be withdrawn from the store depending on the badges which have been presented. 
    /// 
    /// # Example:
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
        /// This function allows creates a new badge which is used for administration purposes and then creates a new 
        /// tiered fund component with the admin badge being able to reset 
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
            tokens_resource_address: ResourceAddress
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
                    tokens_resource_address
                ),
                admin_badge
            )
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
        ///     * When resetting the tier history.
        ///     * For the addition of new tiers.
        ///     * For the removal of tiers.
        ///     * For the modification of tier limits.
        /// * `tokens_resource_address` (ResourceAddress) - The resource address of the tokens that will be used for the
        /// fund.
        /// 
        /// # Returns:
        /// 
        /// * `ComponentAddress` - The component address of the newly instantiated tiered fund.
        pub fn instantiate_custom_tiered_fund(
            administration_rule: AccessRule,
            tokens_resource_address: ResourceAddress
        ) -> ComponentAddress {
            // Performing the checks to determine if the tiered fund can be created
            assert!(
                !matches!(borrow_resource_manager!(tokens_resource_address).resource_type(), ResourceType::NonFungible),
                "[Custom Instantiation]: A tiered fund can not be created with a non-fungible token."
            );

            // At this point we know that the creation of the component can go through.

            // Defining the access rules for the component.
            let access_rules: AccessRules = AccessRules::new()
                .method("reset_history", administration_rule.clone())
                .method("add_tier", administration_rule.clone())
                .method("remove_tier", administration_rule.clone())
                .method("modify_tier_limit", administration_rule.clone())
                .default(rule!(allow_all));

            // Instantiating the component and returning its address
            return Self {
                withdraw_limits: HashMap::new(),
                withdraw_history: HashMap::new(),
                vault: Vault::new(tokens_resource_address),
            }
            .instantiate()
            .add_access_check(access_rules)
            .globalize()
        }
    }
}