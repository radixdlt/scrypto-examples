use scrypto::prelude::*;

blueprint! {
    /// This blueprint defines the state and logic involved in a english auction non-fungible token sale. People who
    /// instantiate components from this blueprint, signify their intent at selling their NFT(s) to the highest bidder
    /// within a specific time period.
    ///
    /// This blueprint allows multiple NFTs to be sold at once as a collection instead of requiring that these NFTs
    /// be sold separately. In addition to that, this blueprint allows XRD payments as well as non-XRD payments for
    /// sellers who opt to accept non-XRD tokens.
    struct EnglishAuction {
        /// These are the vaults where the NFTs will be stored. Since this blueprint allows for multiple NFTs to be sold
        /// at once, this HashMap is used to store all of these NFTs with the lazymap key being the resource address of
        /// these NFTs if they are not all of the same _kind_.
        nft_vaults: HashMap<ResourceAddress, Vault>,

        /// Since this is an English Auction, it means that there will be multiple people bidding on the same NFT(s) at
        /// the same time. This lazymaps maps the bidder's badge to a vault which contains the funds that they bid.
        bid_vaults: HashMap<NonFungibleId, Vault>,

        /// After the winner of the bid has been determined, their tokens will be sent to the payment vault which the
        /// seller has access to and can withdraw funds from.
        payment_vault: Vault,

        /// When a bidder makes a bid, they're given a bidder's badge which proves that they've placed a bid in this
        /// component and with a proof of the amount of funds owed to them. If they wish to then cancel their bid or
        /// terminate it, they must present their bidder's badge to the appropriate methods on the an EnglishAuction
        /// component.
        bidders_badge: ResourceAddress,

        /// This is a vault which contains an internal admin badge. This badge is used to manager the bidder badges and
        /// is given the authority to mint more of them, burn them, and update their non-fungible metadata.
        internal_admin_badge: Vault,

        /// This blueprint accepts XRD as well as non-XRD payments. This variable here is the resource address of the
        /// fungible token that will be used for payments to the component.
        accepted_payment_token: ResourceAddress,

        /// This is the ending epoch. When this epoch is reached or exceeded, the auction will be considered done and
        /// if the minimum automatic sale price is reached, each parties will be given their tokens.
        ending_epoch: u64,

        /// The English Auction is stateful and at different states of the auction different actions may or may not be
        /// possible.
        state: AuctionState
    }

    impl EnglishAuction{

        /// Instantiates a new english auction sale for the passed NFTs.
        ///
        /// This function is used to instantiate a new english auction sale for the passed bucket of NFTs. The auction
        /// can be done for a single NFT or a bundle of NFTs which the seller intends to sell together. The tokens
        /// may be sold for XRD or for any other fungible token of the instantiator's choosing.
        ///
        /// This function performs a number of checks before the `EnglishAuction` component is created:
        ///
        /// * **Check 1:** Checks that the passed buckets of tokens are all non-fungible tokens.
        /// * **Check 2:** Checks that the `accepted_payment_token` is a fungible token.
        /// * **Check 3:** Checks that the ending epoch has not yet passed.
        ///
        /// # Arguments:
        ///
        /// * `non_fungible_tokens` (Vec<Bucket>) - A vector of buckets of the non-fungible tokens that the instantiator
        /// wishes to sell.
        /// * `accepted_payment_token` (ResourceAddress) - Payments may be accepted in XRD or non-XRD tokens. This
        /// argument specifies the resource address of the token the instantiator wishes to accept for payment.
        /// * `starting_price` (Decimal) - The starting price of the NFT bundle sale.
        /// * `ending_price` (Decimal) - The ending price of the NFT bundle sale.
        /// * `relative_ending_epoch` (u64) - This is the relative ending epoch, meaning that this value will be added
        /// with the current epoch. This argument controls the rate at which the price of the bundle decreases. When
        /// the ending epoch is reached, the price will reach its minimum that was specified in the arguments.
        ///
        /// # Returns:
        ///
        /// This function returns a tuple which has the following format:
        /// * `ComponentAddress` - A component address of the instantiated `EnglishAuction` component.
        /// * `Bucket` - A bucket containing an ownership badge which entitles the holder to the assets in this
        /// component.
        pub fn instantiate_english_auction_sale(
            non_fungible_tokens: Vec<Bucket>,
            accepted_payment_token: ResourceAddress,
            relative_ending_epoch: u64,
        ) -> (ComponentAddress, Bucket) {
            // Performing checks to ensure that the creation of the component can go through
            assert!(
                !non_fungible_tokens
                    .iter()
                    .any(
                        |x| borrow_resource_manager!(x.resource_address()).resource_type()
                            != ResourceType::NonFungible
                    ),
                "[Instantiation]: Can not perform a sale for fungible tokens."
            );
            assert!(
                borrow_resource_manager!(accepted_payment_token).resource_type()
                    != ResourceType::NonFungible,
                "[Instantiation]: Only payments of fungible resources are accepted."
            );
            assert!(
                relative_ending_epoch > Runtime::current_epoch(),
                "[Instantiation]: The ending epoch has already passed."
            );

            // At this point we know that the component creation can go through.

            // Create a new HashMap of vaults and aggregate all of the tokens in the buckets into the vaults of this
            // HashMap. This means that if somebody passes multiple buckets of the same resource, then they would end
            // up in the same vault.
            let mut nft_vaults: HashMap<ResourceAddress, Vault> = HashMap::new();
            for bucket in non_fungible_tokens.into_iter() {
                nft_vaults
                    .entry(bucket.resource_address())
                    .or_insert(Vault::new(bucket.resource_address()))
                    .put(bucket)
            }

            // When the owner of the NFT(s) instantiates a new english auction sale component, their tokens are taken away
            // from them and they're given an ownership NFT which is used to authenticate them and as proof of ownership
            // of the NFTs. This ownership badge can be used to either withdraw the funds from the token sale or the
            // NFTs if the seller is no longer interested in selling their tokens.
            let ownership_badge: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Ownership Badge")
                .metadata(
                    "description",
                    "An ownership badge used to authenticate the owner of the NFT(s).",
                )
                .metadata("symbol", "OWNER")
                .initial_supply(1);

            // Creating the internal admin badge which will be used to manager the bidder badges
            let internal_admin_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Internal Admin Badge")
                .metadata("description", "A badge used to manage the bidder badges")
                .metadata("symbol", "IADMIN")
                .initial_supply(1);

            // Creating the bidder's badge which will be used to track the bidder's information and bids.
            let bidder_badge_resource_address: ResourceAddress =
                ResourceBuilder::new_non_fungible()
                    .metadata("name", "Bidder Badge")
                    .metadata(
                        "description",
                        "A badge provided to bidders to keep track of the amount they've bid",
                    )
                    .metadata("symbol", "BIDDER")
                    .mintable(
                        rule!(require(internal_admin_badge.resource_address())),
                        LOCKED,
                    )
                    .burnable(
                        rule!(require(internal_admin_badge.resource_address())),
                        LOCKED,
                    )
                    .updateable_non_fungible_data(
                        rule!(require(internal_admin_badge.resource_address())),
                        LOCKED,
                    )
                    .no_initial_supply();

            // Setting up the access rules for the component methods such that only the owner of the ownership badge can
            // make calls to the protected methods.
            let access_rule: AccessRule = rule!(require(ownership_badge.resource_address()));
            let access_rules: AccessRules = AccessRules::new()
                .method("cancel_auction", access_rule.clone())
                .method("withdraw_payment", access_rule.clone())
                .default(rule!(allow_all));

            // Instantiating the english auction sale component
            let english_auction_sale: ComponentAddress = Self {
                nft_vaults,
                bid_vaults: HashMap::new(),
                payment_vault: Vault::new(accepted_payment_token),
                bidders_badge: bidder_badge_resource_address,
                internal_admin_badge: Vault::with_bucket(internal_admin_badge),
                accepted_payment_token,
                ending_epoch: Runtime::current_epoch() + relative_ending_epoch,
                state: AuctionState::Open,
            }
            .instantiate()
            .add_access_check(access_rules)
            .globalize();

            return (english_auction_sale, ownership_badge);
        }

        /// Attempts to transition the state from Open to Settled.
        ///
        /// The transition of state from `Open` to `Settled` happens when enough epochs pass; therefore, there is no
        /// easy way of triggering the state transition. Therefore, this method is designed to attempt to perform the
        /// state transition if it's currently possible (i.e. we're in the open state and the ending epoch has been
        /// reached.). The idea is that this method call would happen at the beginning of all method calls to ensure
        /// that the state transition takes place and that the winner is settled on the first time anybody calls the
        /// component the first time after the end epoch has passed.
        ///
        /// This method performs no assertions (i.e. rust type assertions which panic) nor should it ever perform that.
        pub fn ensure_auction_settlement(&mut self) {
            match self.state {
                AuctionState::Open if Runtime::current_epoch() > self.ending_epoch => {
                    // We would like to either transition to the Settled state if there are people who have placed bids
                    // and we can select a winner, or transition to the canceled state if there are no bids and the NFTs
                    // should be sent back.
                    if self.has_bids() {
                        // Determining the NFT ID which corresponds to the largest bid that has been made for this NFT
                        // bundle.
                        let non_fungible_id: NonFungibleId = self
                            .bid_vaults
                            .iter()
                            .max_by(|a, b| a.1.amount().cmp(&b.1.amount()))
                            .map(|(k, _v)| k)
                            .unwrap()
                            .clone();

                        // Update the bidder's badge associated with the above non-fungible id to reflect that this is
                        // the winner of the bid.
                        self.internal_admin_badge.authorize(|| {
                            let resource_manager: &ResourceManager =
                                borrow_resource_manager!(self.bidders_badge);

                            // Getting the already existing NFT data of the bidder's badge to update it
                            let mut bidders_badge_data: BidderBadge =
                                resource_manager.get_non_fungible_data(&non_fungible_id);
                            bidders_badge_data.is_winner = true;

                            // Setting the new data as the data of that badge
                            resource_manager.update_non_fungible_data(&non_fungible_id, bidders_badge_data);
                        });

                        // Take the funds from the winner's vault and put them in the payment vault so that the seller
                        // can now withdraw them
                        self.payment_vault.put(
                            self.bid_vaults
                                .get_mut(&non_fungible_id)
                                .unwrap()
                                .take_all(),
                        );
                    } else {
                        self.state = AuctionState::Canceled
                    }
                },
                _ => {}
            }
        }

        /// Checks if the NFT bundle has any bids.
        ///
        /// Returns:
        ///
        /// `bool` - A boolean of whether this NFT bundle has any bids or not. If this method returns `true` then there
        /// are bids on the NFT bundle, otherwise if `false` is returned then it means that there are no bids.
        pub fn has_bids(&self) -> bool {
            return self.bid_vaults.len() > 0
        }
    }
}

/// The data used for the bidder's non-fungible token. This NFT is used to authenticate bidders and allow them to add to
/// their bids, cancel them, or claim the NFT bundle if they win the bid.
#[derive(NonFungibleData)]
struct BidderBadge {
    /// A mutable decimal which holds information on the amount of funds that this bidder has bid. This is mutable as
    /// bidders are allowed to add to their bid.
    #[scrypto(mutable)]
    bid_amount: Decimal,

    /// A boolean which holds information on whether this bidder is the winner of the bid or not.
    #[scrypto(mutable)]
    is_winner: bool,
}

/// The English auction is by definition stateful and during different periods and states of the auction different
/// actions may be allowed or disallowed. This enum describes the state of the English auction component.
#[derive(Encode, Decode, TypeId, Describe)]
enum AuctionState {
    Open,
    Settled,
    Canceled,
}
