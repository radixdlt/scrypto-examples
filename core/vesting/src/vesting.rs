use crate::beneficiary::BeneficiaryVestingSchedule;
use scrypto::prelude::*;

#[blueprint]
mod vesting {
    // Setting up the auth for the vesting component. With v0.4.0 of Scrypto we can now make the authentication
    // and authorization to happen automatically without us needing to care about them. We can use this to
    // impose quite a number of rules on who is authorized to access what.
    // let rules = AccessRulesConfig::new()
    enable_method_auth! {
        roles {
            admin => updatable_by: [SELF]; 
        },
        methods {
            // Only people who have at least 1 admin badge in their auth zone may make calls to these methods.
            add_beneficiary => restrict_to: [admin];
            // Only transactions where a minimum of `min_admins_required_for_multi_admin` admin badges are present
            // in the auth zone are allowed to make calls to these methods. This makes these methods dynamic as this
            // value will change as admins are added.
            terminate_beneficiary => restrict_to: [admin];
            add_admin => restrict_to: [admin];
            disable_termination => restrict_to: [admin];
            withdraw_funds => PUBLIC;
        }
    }
    /// The vesting blueprint allows for a vesting schedule to be setup whereby "beneficiaries" are given tokens over a
    /// period of time with a specific cliff and vesting period. The vesting blueprint follows a linear graph to vesting
    /// whereby no tokens are vested between the enrollment and the cliff epoch. Then, from the cliff epoch all the way
    /// the end epoch the vesting happens in a linear manner beginning from the specified amount to be unlocked at cliff
    /// and ending with the final amount.
    ///
    /// There are two main parties in the vesting blueprint:
    /// * An Admin: The admin is any party which has the `admin_badge`, typically this would be the instantiator of the
    /// component but it does not have to be. The admin pays the tokens and and sets the vesting schedule (or has that
    /// done on their behalf) and has the right to terminate the vesting of tokens at any period if they so choose to.
    /// * A Beneficiary: Any party which has a valid vesting schedule non-fungible token provided by the vesting
    /// component. The vesting schedule provides the beneficiaries with the right to withdraw their tokens from the
    /// vesting component, authenticates them, and keeps track of the important information used to track their amount
    /// of vested tokens. The vesting blueprint allows for the existence of multiple beneficiaries enrolled in the same
    /// component.
    ///
    /// The termination of the vesting of tokens at any point of time without prior notice can be of worry especially
    /// to people waiting for their funds to vest. Therefore, the vesting blueprint allows for the admin to give up
    /// their termination rights which may be needed in some cases. In a vesting component with multiple admins, the
    /// majority of the admins are required to agree to agree to the giving up of termination rights before they're
    /// given up.
    ///
    /// An interactive version of the vesting graph may be found [here](https://www.desmos.com/calculator/7qetg3g31f)
    /// where you can modify how long the periods are as well as how much funds we're vesting to see what the graph
    /// for that would look like.
    struct Vesting {
        /// A HashMap which maps the non-fungible ids of beneficiaries and the vaults associated with them. Meaning that
        /// each beneficiary has their own vault where their un-vested funds are stored.
        funds: HashMap<NonFungibleLocalId, Vault>,

        /// The beneficiary is given a badge to be able to authenticate them later on and to keep track of the amount of
        /// funds owed to them by the component at a given epoch. The badge given to beneficiaries is a vesting schedule
        /// badge which keeps track of their vesting schedule.
        beneficiary_vesting_badge: ResourceManager,

        /// An admin badge which is returned after the vesting component is created. The admin badge has the right to
        /// terminate the vesting schedule at any point of time for any external reason.
        admin_badge: ResourceManager,

        /// A vector of the dead vaults which are no longer being used by the vesting component. The dead vaults are
        /// usually there after a beneficiary has been terminated and we need to take their empty vault and put it away.
        dead_vaults: Vec<Vault>,

        /// There are certain operations in the vesting blueprint that require that multiple admins approve the
        /// operation in order to allow it to go through. As an example, in order for the admins to give up their
        /// termination rights a majority of the admins must agree that the termination rights should be given up.
        /// Otherwise, the `giveup_admin_rights` function won't be callable. Similarity, if any admin can simply add
        /// another admin, then they can have an advantage when it comes to decisions. Therefore, the adding of admins
        /// is an operation that can only be done when a majority of the admins have agreed to do it.
        min_admins_required_for_multi_admin: Decimal,

        /// A boolean which controls whether the admin can terminate beneficiary's vesting schedules or not.
        admin_may_terminate: bool,
    }

    impl Vesting {
        /// Creates a new vesting component
        ///
        /// This function is used to create a new vesting component. The main purpose of this function is to setup the
        /// auth of the vesting component in the way that is needed and to create the needed resources for the badges.
        /// This method does not perform any checks when before creating the vesting component.
        ///
        /// # Returns:
        ///
        /// Returns a `(ComponentAddress, Bucket)` tuple of the following format:
        ///
        /// * `ComponentAddress` - The address of the newly instantiated vesting component.
        /// * `Bucket` - A bucket containing the admin badge for the vesting component.
        pub fn instantiate_vesting() -> (Global<Vesting>, FungibleBucket) {
            // Creating the Actor Virtual Badge which we will give authority to mint and burn the admin and beneficiary
            // badges.
            let (address_reservation, component_address) = 
                Runtime::allocate_component_address(Vesting::blueprint_id());

            // Creating the admin badge and setting its auth. The admin badge may be burned by the internal admin badge
            // in the caste of the admin giving up their termination rights
            let admin_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata!(
                    init {
                        "name" => "Vesting Admin Badge".to_owned(), locked;
                        "description" => 
                        "An admin badge with the authority to terminate the vesting of tokens".to_owned(), locked;
                    }
                ))
                .mint_roles(mint_roles! {
                        minter => rule!(require(global_caller(component_address)));
                        minter_updater => rule!(deny_all);
                    }
                )
                .mint_initial_supply(dec!("1"));
            

            // Creating the beneficiary's badge which is used to keep track of their vesting schedule.
            let beneficiary_vesting_badge: ResourceManager = ResourceBuilder::new_integer_non_fungible::<BeneficiaryVestingSchedule>(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "Beneficiary Badge".to_string(), locked;
                        "description" => 
                        "A badge provided to beneficiaries by the vesting component for authentication".to_string(), locked;
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(deny_all); 
                ))
                .create_with_no_initial_supply();

            let vesting_component = Self {
                funds: HashMap::new(),
                beneficiary_vesting_badge: beneficiary_vesting_badge,
                admin_badge: admin_badge.resource_manager(),
                dead_vaults: Vec::new(),
                admin_may_terminate: true,
                min_admins_required_for_multi_admin: dec!("1"),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(address_reservation)
            .roles(
                roles!(
                    admin => rule!(
                        require_amount(
                            dec!(1),
                            admin_badge.resource_address()
                        )
                    );
                )
            )
            .globalize();
    
            return (vesting_component, admin_badge);
        }

        /// Adds a new beneficiary to to the vesting component.
        ///
        /// This is an authenticated method which may only be called by an admin. This method is used to add a new
        /// beneficiary to the vesting component which has the a vesting schedule according to the arguments passed.
        ///
        /// This method performs a number of checks before instantiating a new `Vesting` component:
        ///
        /// * **Check 1:** Checks that the funds provided are fungible and not non-fungible.
        /// * **Check 2:** Checks that the passed bucket of funds is not an empty bucket.
        ///
        /// * `funds` (Bucket) - A bucket of funds which we wish to vest over a period of time.
        /// * `relative_cliff_epoch` (u64) - Defines the number of epochs in the future where the cliff will happen.
        /// this is a relative epoch (relative to the current epoch) and not an absolute epoch.
        /// * `relative_ending_epoch` (u64) - Defines the number of epochs in the future where the vesting will end.
        /// this is a relative epoch (relative to the current epoch) and not an absolute epoch.
        /// * `percentage_available_on_cliff` (Decimal) - A decimal between the numbers 0 and 1 which defines the
        /// percentage of funds available to the beneficiary gets access to when the cliff period ends.
        ///
        /// # Returns
        ///
        /// * `Bucket` - A bucket containing the badge of the beneficiary.
        pub fn add_beneficiary(
            &mut self,
            funds: Bucket,
            relative_cliff_epoch: u64,
            relative_ending_epoch: u64,
            percentage_available_on_cliff: Decimal,
        ) -> Bucket {
            // Performing checks to ensure that the beneficiary may be added.
            match ResourceManager::from_address(funds.resource_address()).resource_type() {
                ResourceType::NonFungible { id_type: _ } => {
                    panic!("[Add Beneficiary]: Can't vest non-fungible tokens for the beneficiary.")
                }
                _ => {}
            }
            assert!(
                !funds.is_empty(),
                "[Add Beneficiary]: Can't vest an empty bucket of funds."
            );

            // At this point we know that the beneficiary may be added to the vesting component, so we go ahead and mint
            // them a non-fungible token with their vesting schedule
            let beneficiary_id: NonFungibleLocalId = NonFungibleLocalId::Integer(
                ((self.funds.len() + self.dead_vaults.len()) as u64 + 1u64).into(),
            );
            let beneficiary_badge: Bucket = self.beneficiary_vesting_badge.mint_non_fungible(
                    &beneficiary_id,
                    BeneficiaryVestingSchedule::new(
                        relative_cliff_epoch,
                        relative_ending_epoch,
                        funds.amount(),
                        percentage_available_on_cliff,
                    ),
                );

            // Putting the funds in a vault to store them int the component
            self.funds.insert(beneficiary_id, Vault::with_bucket(funds));

            // Returning the beneficiary their badge back to them
            return beneficiary_badge;
        }

        /// Terminates the vesting schedule of a given beneficiary.
        ///
        /// This is an authenticated method which may only be called by an admin. This method terminates the vesting of
        /// tokens for a beneficiary with the provided beneficiary id and withdraws their unclaimed tokens back to the
        /// caller of this method.
        ///
        /// This method performs a number of checks before the beneficiary is terminated:
        ///
        /// * **Check 1:** Checks that the passed `beneficiary_id` is a valid id of a current beneficiary.
        /// * **Check 2:** Checks that the admin does have the authority to terminate vesting of tokens.
        ///
        /// # Arguments:
        ///
        /// * `beneficiary_id` (NonFungibleId) - A non-fungible id of the beneficiary's vesting schedule we would like
        /// to terminate.
        pub fn terminate_beneficiary(&mut self, beneficiary_id: NonFungibleLocalId) -> Bucket {
            // Checking that the given beneficiary id belongs to a valid beneficiary
            assert!(
                self.funds.contains_key(&beneficiary_id),
                "[Beneficiary Termination]: Invalid beneficiary id provided."
            );
            assert!(
                self.admin_may_terminate,
                "[Beneficiary Termination]: Admin has given up termination rights and may no longer terminate vesting."
            );

            // Taking the remaining unclaimed amount from the beneficiary's vault
            let unclaimed_funds: Bucket = self.funds.get_mut(&beneficiary_id).unwrap().take_all();

            // Removing the empty vault from the hashmap and into the vaults of dead vaults
            self.dead_vaults
                .push(self.funds.remove(&beneficiary_id).unwrap());

            return unclaimed_funds;
        }

        /// Adds a new admin and calculates the amount of admins required for multi-admin method calls.
        ///
        /// This method is used to mint a new admin badge and then calculate the amount of admins which needs to agree
        /// multi-admin method calls before they're made. One key thing to note is that multi-admin method calls require
        /// a simple majority in order for them to go through.
        ///
        /// # Arguments:
        ///
        /// * `admin_badges_to_mint` (Decimal) - The amount of admin badges which we wish to create.
        ///
        /// # Returns:
        ///
        /// * `Bucket` - A bucket of admin badges.
        pub fn add_admin(&mut self, admin_badges_to_mint: Decimal) -> Bucket {
            // Getting the resource manager of the admin badge
            let admin_resource_manager = self.admin_badge;

            // Minting a new admin badge for the caller
            let admin_badge: Bucket = admin_resource_manager.mint(admin_badges_to_mint);

            // Determining the amount of admins required for a multi-admin call to be made. This number will always be
            // 50% or more depending on the total amount of admin badges.
            self.min_admins_required_for_multi_admin =
                if admin_resource_manager.total_supply().unwrap() <= dec!("2") {
                    admin_resource_manager.total_supply().unwrap()
                } else {
                    (admin_resource_manager.total_supply().unwrap().checked_div(dec!("2")).unwrap().checked_ceiling()).unwrap()
                };
            info!(
                "[Add Admin]: Minimum required admins is: {}",
                self.min_admins_required_for_multi_admin
            );

            // Updating the "admin" Role's AccessRule
            Runtime::global_component()
                .set_role(
                    "admin",
                    rule!(
                        require_amount(
                            self.min_admins_required_for_multi_admin, 
                            admin_badge.resource_address()
                        )
                    )
                );

            // Returning the newly created admin badge back to the caller
            return admin_badge;
        }

        /// Withdraws the funds vested so far for the beneficiary
        ///
        /// This is an authenticated method which can only be called by a beneficiary. This method withdraws the tokens
        /// which have been vested so far from the component and returns them back to the caller of the method.
        ///
        /// This method performs a number of checks before withdrawing the funds.
        ///
        /// * **Check 1:** Checks to ensure that the passed proof does contain a valid beneficiary badge.
        /// * **Check 2:** Checks to ensure that the proof contains exactly one badge.
        /// * **Check 1:** Checks to ensure that the beneficiary has not been terminated.
        ///
        /// # Arguments
        ///
        /// * `beneficiary_badge` (Proof) - A Proof of the beneficiary's badge.
        ///
        /// # Returns
        ///
        /// * `Bucket` - A bucket of the vested tokens.
        pub fn withdraw_funds(&mut self, beneficiary_badge: Proof) -> Bucket {
            // Checking that the funds may be withdrawn from the component
            let beneficiary_badge = beneficiary_badge
                .check(self.beneficiary_vesting_badge.address());
                
            let beneficiary_ids: Vec<NonFungibleLocalId> = beneficiary_badge
                .as_non_fungible()
                .non_fungible_local_ids()
                .into_iter()
                .collect::<Vec<NonFungibleLocalId>>();

            let beneficiary_id: NonFungibleLocalId = beneficiary_ids[0].clone();

            assert!(
                self.funds.contains_key(&beneficiary_id),
                "[Withdraw Funds]: Vesting has been terminated. Contact your admin for more information."
            );

            // At this point we're sure that the withdraw may go through
            let beneficiary_vesting_schedule: BeneficiaryVestingSchedule =
                self.beneficiary_vesting_badge
                    .get_non_fungible_data::<BeneficiaryVestingSchedule>(&beneficiary_id);

            // The amount that we should return back is the difference between the amount of funds in the vault right
            // now and the amount that should have not have vested yet.
            let beneficiary_vault: &mut Vault = self.funds.get_mut(&beneficiary_id).unwrap();
            let claim_amount: Decimal = beneficiary_vault.amount()
                .checked_sub(beneficiary_vesting_schedule.get_unvested_amount(Runtime::current_epoch().number())).unwrap();
            info!(
                "[Withdraw Funds]: Withdraw successful. Withdrawing {} tokens",
                claim_amount
            );
            return beneficiary_vault.take(claim_amount);
        }

        /// Disables the termination of vesting schedules globally across all admins.
        ///
        /// This is an authenticated method which may only be called by admins. When this method is called, termination
        /// of vesting schedules is disabled for all admins.
        pub fn disable_termination(&mut self) {
            self.admin_may_terminate = false;
        }
    }
}


