use scrypto::prelude::*;

#[blueprint]
mod english_auction {
    // Setting up the access rules for the component methods such that only the owner of the ownership badge can
    // make calls to the protected methods.
    enable_method_auth! {
        methods {
            cancel_auction => restrict_to: [OWNER];
            withdraw_payment => restrict_to: [OWNER];
            bid => PUBLIC;
            increase_bid => PUBLIC;
            cancel_bid => PUBLIC;
            claim_nfts => PUBLIC;
            ensure_auction_settlement => PUBLIC;
            has_bids => PUBLIC;
        }
    }
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
        nft_vaults: HashMap<ResourceAddress, NonFungibleVault>,

        /// Since this is an English Auction, it means that there will be multiple people bidding on the same NFT(s) at
        /// the same time. This lazymaps maps the bidder's badge to a vault which contains the funds that they bid.
        bid_vaults: HashMap<NonFungibleLocalId, Vault>,

        /// After the winner of the bid has been determined, their tokens will be sent to the payment vault which the
        /// seller has access to and can withdraw funds from.
        payment_vault: Vault,

        /// When a bidder makes a bid, they're given a bidder's badge which proves that they've placed a bid in this
        /// component and with a proof of the amount of funds owed to them. If they wish to then cancel their bid or
        /// terminate it, they must present their bidder's badge to the appropriate methods on the an EnglishAuction
        /// component.
        bidders_badge: ResourceManager,

        /// This blueprint accepts XRD as well as non-XRD payments. This variable here is the resource address of the
        /// fungible token that will be used for payments to the component.
        accepted_payment_token: ResourceAddress,

        /// This is the ending epoch. When this epoch is reached or exceeded, the auction will be considered done and
        /// if the minimum automatic sale price is reached, each parties will be given their tokens.
        ending_epoch: Epoch,

        /// The English Auction is stateful and at different states of the auction different actions may or may not be
        /// possible.
        state: AuctionState,
    }

    impl EnglishAuction {
        // =============================================================================================================
        // The following are methods which only the seller (auctioneer) needs and can call.
        // =============================================================================================================

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
        /// * `Global<EnglishAuction>` - A Global<EnglishAuction> component object of the instantiated `EnglishAuction` component.
        /// * `Bucket` - A bucket containing an ownership badge which entitles the holder to the assets in this
        /// component.
        pub fn instantiate_english_auction(
            non_fungible_tokens: Vec<NonFungibleBucket>,
            accepted_payment_token: ResourceAddress,
            relative_ending_epoch: u64,
        ) -> (Global<EnglishAuction>, FungibleBucket) {
            // Performing checks to ensure that the creation of the component can go through
            // assert!(
            //     !non_fungible_tokens.iter().any(|x| !matches!(
            //         ResourceManager::from_address(x.resource_address()).resource_type(),
            //         ResourceType::NonFungible { id_type: _ }
            //     )),
            //     "[Instantiation]: Can not perform a sale for fungible tokens."
            // );
            assert!(
                !matches!(
                    ResourceManager::from_address(accepted_payment_token).resource_type(),
                    ResourceType::NonFungible { id_type: _ }
                ),
                "[Instantiation]: Only payments of fungible resources are accepted."
            );
            assert!(
                Runtime::current_epoch().after(relative_ending_epoch).unwrap() > Runtime::current_epoch(),
                "[Instantiation]: The ending epoch has already passed."
            );

            // At this point we know that the component creation can go through.

            // Create a new HashMap of vaults and aggregate all of the tokens in the buckets into the vaults of this
            // HashMap. This means that if somebody passes multiple buckets of the same resource, then they would end
            // up in the same vault.
            let mut nft_vaults: HashMap<ResourceAddress, NonFungibleVault> = HashMap::new();
            for bucket in non_fungible_tokens.into_iter() {
                nft_vaults
                    .entry(bucket.resource_address())
                    .or_insert(NonFungibleVault::new(bucket.resource_address()))
                    .put(bucket)
            }

            // When the owner of the NFT(s) instantiates a new english auction sale component, their tokens are taken away
            // from them and they're given an ownership NFT which is used to authenticate them and as proof of ownership
            // of the NFTs. This ownership badge can be used to either withdraw the funds from the token sale or the
            // NFTs if the seller is no longer interested in selling their tokens.
            let ownership_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "Ownership Badge".to_owned(), locked;
                        "description" =>
                        "An ownership badge used to authenticate the owner of the NFT(s).".to_owned(), locked;
                        "symbol" => "OWNER".to_owned(), locked;
                    }
                ))
                .mint_initial_supply(1);

            // Creating the internal admin badge which will be used to manager the bidder badges
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(EnglishAuction::blueprint_id());

            // Creating the bidder's badge which will be used to track the bidder's information and bids.
            let bidder_badge_resource_address: ResourceManager =
                ResourceBuilder::new_ruid_non_fungible::<BidderBadge>(OwnerRole::None)
                    .metadata(metadata!(
                        init {
                            "name" => "Bidder Badge".to_owned(), locked;
                            "description" =>
                            "A badge provided to bidders to keep track of the amount they've bid".to_owned(), locked;
                            "symbol" => "BIDDER".to_owned(), locked;
                        }
                    ))
                    .mint_roles(mint_roles!(
                        minter => rule!(require(global_caller(component_address)));
                        minter_updater => rule!(deny_all);
                    ))
                    .burn_roles(burn_roles!(
                        burner => rule!(require(global_caller(component_address)));
                        burner_updater => rule!(deny_all);
                    ))
                    .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                        non_fungible_data_updater => rule!(require(global_caller(component_address)));
                        non_fungible_data_updater_updater => rule!(deny_all);
                    ))
                    .create_with_no_initial_supply();

            // let access_rule: AccessRule = rule!(require(ownership_badge.resource_address()));
            // let access_rules = AccessRulesConfig::new()
            //     .method("cancel_auction", access_rule.clone(), AccessRule::DenyAll)
            //     .method("withdraw_payment", access_rule.clone(), AccessRule::DenyAll)
            //     .default(rule!(allow_all), AccessRule::DenyAll);

            // Instantiating the english auction sale component
            let english_auction = Self {
                nft_vaults,
                bid_vaults: HashMap::new(),
                payment_vault: Vault::new(accepted_payment_token),
                bidders_badge: bidder_badge_resource_address,
                accepted_payment_token,
                ending_epoch: Runtime::current_epoch().after(relative_ending_epoch).unwrap(),
                state: AuctionState::Open,
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::Updatable(rule!(require(
                ownership_badge.resource_address()
            ))))
                .with_address(address_reservation)
                .globalize();

            return (english_auction, ownership_badge);
        }

        /// Cancels the auctioning of tokens and returns them back to their owner.
        ///
        /// This method performs a single check before canceling the auction.
        ///
        /// * **Check 1:** Checks that the auction is either `Open` or `Canceled`.
        ///
        /// # Returns:
        ///
        /// * `Vec<Bucket>` - A vector of buckets of the non-fungible tokens which were being sold.
        ///
        /// # Note:
        ///
        /// * This is an authenticated method which may only be called by the holder of the `ownership_badge`.
        pub fn cancel_auction(&mut self) -> Vec<NonFungibleBucket> {
            // Mandatory call to ensure that the `ensure_auction_settlement` method to ensure that if the conditions are
            // met, that the auction will proceed to the next stage/state.
            self.ensure_auction_settlement();

            // Checking if the auction can be canceled.
            assert!(
                matches!(self.state, AuctionState::Open)
                    || matches!(self.state, AuctionState::Canceled),
                "[Cancel Auction]: Can not cancel the auction unless we're still "
            );

            // At this point we know that the auction can be canceled. So, we withdraw the NFTs and return them to the
            // caller
            self.state = AuctionState::Canceled;

            let resource_addresses: Vec<ResourceAddress> =
                self.nft_vaults.keys().cloned().collect();
            let mut tokens: Vec<NonFungibleBucket> = Vec::new();
            for resource_address in resource_addresses.into_iter() {
                tokens.push(
                    self.nft_vaults
                        .get_mut(&resource_address)
                        .unwrap()
                        .take_all(),
                )
            }

            return tokens;
        }

        /// Withdraws the payment owed from the sale.
        ///
        /// This method performs a single check before canceling the sale:
        ///
        /// * **Check 1:** Checks that the auction is settled.
        ///
        /// # Returns:
        ///
        /// * `Bucket` - A bucket containing the payment.
        ///
        /// # Note:
        ///
        /// * This is an authenticated method which may only be called by the holder of the `ownership_badge`.
        /// * There is no danger in not checking if the sale has occurred or not and attempting to return the tokens
        /// anyway. If we do not have the payment tokens then the worst case scenario would be that an empty bucket is
        /// returned. This is bad from a UX point of view but does not pose any security risk.
        pub fn withdraw_payment(&mut self) -> Bucket {
            // Mandatory call to ensure that the `ensure_auction_settlement` method to ensure that if the conditions are
            // met, that the auction will proceed to the next stage/state.
            self.ensure_auction_settlement();

            // Checking if the payment can be withdrawn
            assert!(
                matches!(self.state, AuctionState::Settled),
                "[Withdraw Payment]: The payment can only be withdrawn when the auction is settled"
            );

            // At this point we know that the payment can be withdrawn
            return self.payment_vault.take_all();
        }

        // =============================================================================================================
        // The following are methods which only bidders need and can call.
        // =============================================================================================================

        /// Allows the caller to Bid in this auction.
        ///
        /// This method allows the caller to bid in this auction with their funds. This method will lock up the bidding
        /// funds and provide the caller with a Bidder's badge in return. The locked up funds can be unlocked at any
        /// point before the auction ends without any issues. However, after the auction ends, the locked up funds may
        /// only be unlocked if the bidder did not win the bid.
        ///
        /// This method performs a number of checks before the bid can be made:
        ///
        /// * **Check 1:** Checks that the auction is in the `Open` state.
        ///
        /// # Arguments:
        ///
        /// * `funds` (Bucket) - A bucket of the funds to bid.
        ///
        /// # Returns:
        ///
        /// * `Bucket` - A bucket of the bidder's badge.
        pub fn bid(&mut self, funds: Bucket) -> Bucket {
            // Mandatory call to ensure that the `ensure_auction_settlement` method to ensure that if the conditions are
            // met, that the auction will proceed to the next stage/state.
            self.ensure_auction_settlement();

            // Performing checks to ensure that the bid can be added
            assert!(
                matches!(self.state, AuctionState::Open),
                "[Bid]: Bids may only be added while the auction is open."
            );

            // At this point we know that a bid can be added.

            // Issuing a bidder's NFT to this bidder with information on the amount that they're bidding

            let bidders_badge: Bucket = self.bidders_badge.mint_ruid_non_fungible(BidderBadge {
                bid_amount: funds.amount(),
                is_winner: false,
            });

            let non_fungible_local_id: NonFungibleLocalId =
                bidders_badge.as_non_fungible().non_fungible_local_id();

            // Taking the bidder's funds and depositing them into a newly created vault where their funds will now live
            self.bid_vaults
                .insert(non_fungible_local_id, Vault::with_bucket(funds));

            // Returning the bidder's badge back to the caller
            return bidders_badge;
        }

        /// Allows a bidder to increase their Bid.
        ///
        /// This is an authenticated which which allows bidders to increase the amount that they are bidding in the
        /// auction to increase their chances of winning the bid.
        ///
        /// This method performs a number of checks before the bid can be increased:
        ///
        /// * **Check 1:** Checks that the auction is in the `Open` state.
        /// * **Check 2:** Checks that the payment was provided in the required token.
        /// * **Check 3:** Checks that the badge provided is a valid bidder's badge.
        /// * **Check 4:** Checks that the `Proof` contains a single bidder's badge.
        ///
        /// # Arguments:
        ///
        /// * `funds` (Bucket) - A bucket of the funds the bidder wishes to add to their bid.
        /// * `bidders_badge` (Proof) - A `Proof` of the bidder's badge.
        pub fn increase_bid(&mut self, funds: Bucket, bidders_badge: Proof) {
            // Mandatory call to ensure that the `ensure_auction_settlement` method to ensure that if the conditions are
            // met, that the auction will proceed to the next stage/state.
            self.ensure_auction_settlement();

            // Checking if the bid can be increased or not.
            assert!(
                matches!(self.state, AuctionState::Open),
                "[Increase Bid]: Bids may only be increased while the auction is open."
            );
            assert_eq!(
                funds.resource_address(),
                self.accepted_payment_token,
                "[Increase Bid]: Invalid tokens were provided as bid. Bids are only allowed in {:?}",
                self.accepted_payment_token
            );
            let bidders_badge = bidders_badge.check(self.bidders_badge.address());

            assert_eq!(
                bidders_badge.amount(), Decimal::one(),
                "[Increase Bid]: This method requires that exactly one bidder's badge is passed to the method"
            );

            // At this point we know that the bidder's bid can be increased.

            // Updating the metadata of the bidder's badge to reflect on the update of the bidder's bid
            let bidders_badge_data: BidderBadge =
                bidders_badge.as_non_fungible().non_fungible().data();
            let non_fungible_local_id: NonFungibleLocalId =
                bidders_badge.as_non_fungible().non_fungible_local_id();
            let resource_manager = self.bidders_badge;
            resource_manager.update_non_fungible_data(
                &non_fungible_local_id,
                "bid_amount",
                bidders_badge_data.bid_amount.checked_add(funds.amount()),
            );

            // Adding the funds to the vault of the bidder
            self.bid_vaults
                .get_mut(
                    &bidders_badge
                        .as_non_fungible()
                        .non_fungible::<BidderBadge>()
                        .local_id(),
                )
                .unwrap()
                .put(funds);
        }

        /// Allows bidders to cancel their bids.
        ///
        /// This method allows bidders to cancel their bids, burning their badge in the process and taking out the funds
        /// which were locked up in the component for the bid.
        ///
        /// A bid may be canceled in one of the following situations:
        ///
        /// 1. Before the auction has ended if the bidder is no longer interested in bidding in this auction.
        /// 2. After the end of the auction if the bidder is not the winner of the auction.
        ///
        /// This method performs a number of checks before the bid is cancelled
        ///
        /// * **Check 1:** Checks that the badge provided is a valid bidder's badge.
        /// * **Check 2:** Checks that the `Proof` contains a single bidder's badge.
        /// * **Check 3:** Checks that the badge provided is not the winner's badge.
        ///
        /// # Arguments:
        ///
        /// * `bidders_badge` (Bucket) - A `Bucket` of the bidder's badge.
        ///
        /// # Returns:
        ///
        /// * `Bucket` - A bucket of the funds owed to the bidder.
        pub fn cancel_bid(&mut self, bidders_badge: Bucket) -> Bucket {
            // Mandatory call to ensure that the `ensure_auction_settlement` method to ensure that if the conditions are
            // met, that the auction will proceed to the next stage/state.
            self.ensure_auction_settlement();

            // Checking if the bid can be canceled or not.
            // assert_eq!(
            //     bidders_badge.resource_address(),
            //     self.bidders_badge,
            //     "[Cancel Bid]: Badge provided is not a valid bidder's badge"
            // );
            assert_eq!(
                bidders_badge.amount(), Decimal::one(),
                "[Cancel Bid]: This method requires that exactly one bidder's badge is passed to the method"
            );
            assert!(
                !bidders_badge
                    .as_non_fungible()
                    .non_fungible::<BidderBadge>()
                    .data()
                    .is_winner,
                "[Cancel Bid]: You can not cancel your bid after winning the auction."
            );
            // At this point we know that the bid cancellation can go on.
            // Take out the bidder's funds from their vault
            let funds: Bucket = self
                .bid_vaults
                .get_mut(
                    &bidders_badge
                        .as_non_fungible()
                        .non_fungible::<BidderBadge>()
                        .local_id(),
                )
                .unwrap()
                .take_all();
            // This bidder will no longer need their badge. We can now safely burn the badge.
            bidders_badge.burn();
            // The bidder's funds may now be returned to them
            return funds;
        }

        /// Allows the winning bidder to claim their NFTs.
        ///
        /// This is a method which allows the winning bidder to claim their NFTs from the component. This method
        /// requires that the bidder passes a bucket with their winning bidder's badge which is burned and in exchange
        /// for that, the caller is given the NFTs which had been locked up in this component.
        ///
        /// This method performs a number of checks before unlocking the NFTs:
        ///
        /// * **Check 1:** Checks that the auction state is `Settled`.
        /// * **Check 2:** Checks that the badge provided is a valid bidder's badge.
        /// * **Check 3:** Checks that the `Proof` contains a single bidder's badge.
        /// * **Check 4:** Checks that the badge provided is the winner's badge.
        ///
        /// # Arguments:
        ///
        /// * `bidders_badge` (Bucket) - A `Bucket` of the bidder's badge.
        ///
        /// # Returns:
        ///
        /// * `Vec<Bucket>` - A vector of buckets of the non-fungible tokens which were being auctioned.
        pub fn claim_nfts(&mut self, bidders_badge: Bucket) -> Vec<NonFungibleBucket> {
            // Mandatory call to ensure that the `ensure_auction_settlement` method to ensure that if the conditions are
            // met, that the auction will proceed to the next stage/state.
            self.ensure_auction_settlement();

            // Checking if the NFTs in this component can be claimed
            assert!(
                matches!(self.state, AuctionState::Settled),
                "[Claim NFTs]: NFTs can only be claimed when the auction has settled."
            );

            // assert_eq!(
            //     bidders_badge.resource_address(),
            //     self.bidders_badge,
            //     "[Claim NFTs]: Badge provided is not a valid bidder's badge"
            // );
            assert_eq!(
                bidders_badge.amount(), Decimal::one(),
                "[Claim NFTs]: This method requires that exactly one bidder's badge is passed to the method"
            );
            assert!(
                bidders_badge
                    .as_non_fungible()
                    .non_fungible::<BidderBadge>()
                    .data()
                    .is_winner,
                "[Claim NFTs]: Badge provided is not the winner's badge."
            );

            // At this point we know that the NFTs can be claimed from the component

            // We can safely burn the bidder's badge at this point as it is no longer needed by the bidder.
            bidders_badge.burn();

            // Getting all of the NFTs from the auction and returning them to the caller
            let resource_addresses: Vec<ResourceAddress> =
                self.nft_vaults.keys().cloned().collect();
            let mut tokens: Vec<NonFungibleBucket> = Vec::new();
            for resource_address in resource_addresses.into_iter() {
                tokens.push(
                    self.nft_vaults
                        .get_mut(&resource_address)
                        .unwrap()
                        .take_all(),
                )
            }

            return tokens;
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
                AuctionState::Open if Runtime::current_epoch() >= self.ending_epoch => {
                    // We would like to either transition to the Settled state if there are people who have placed bids
                    // and we can select a winner, or transition to the canceled state if there are no bids and the NFTs
                    // should be sent back.
                    if self.has_bids() {
                        // Determining the NFT ID which corresponds to the largest bid that has been made for this NFT
                        // bundle.
                        let non_fungible_local_id: NonFungibleLocalId = self
                            .bid_vaults
                            .iter()
                            .max_by(|a, b| a.1.amount().cmp(&b.1.amount()))
                            .map(|(k, _v)| k)
                            .unwrap()
                            .clone();

                        // Update the bidder's badge associated with the above non-fungible id to reflect that this is
                        // the winner of the bid.
                        let resource_manager = self.bidders_badge;

                        // Setting the new data as the data of that badge
                        resource_manager.update_non_fungible_data(
                            &non_fungible_local_id,
                            "is_winner",
                            true,
                        );

                        // Take the funds from the winner's vault and put them in the payment vault so that the seller
                        // can now withdraw them
                        self.payment_vault.put(
                            self.bid_vaults
                                .get_mut(&non_fungible_local_id)
                                .unwrap()
                                .take_all(),
                        );

                        self.state = AuctionState::Settled
                    } else {
                        self.state = AuctionState::Canceled
                    }
                }
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
            return self.bid_vaults.len() > 0;
        }
    }
}

/// The data used for the bidder's non-fungible token. This NFT is used to authenticate bidders and allow them to add to
/// their bids, cancel them, or claim the NFT bundle if they win the bid.
#[derive(NonFungibleData, ScryptoSbor)]
struct BidderBadge {
    /// A mutable decimal which holds information on the amount of funds that this bidder has bid. This is mutable as
    /// bidders are allowed to add to their bid.
    #[mutable]
    bid_amount: Decimal,

    /// A boolean which holds information on whether this bidder is the winner of the bid or not.
    #[mutable]
    is_winner: bool,
}

/// The English auction is by definition stateful and during different periods and states of the auction different
/// actions may be allowed or disallowed. This enum describes the state of the English auction component.
#[derive(Debug, ScryptoSbor)]
enum AuctionState {
    /// An auction is said to be open if the end epoch of the auction has not yet passed and if the seller has not
    /// decided to cancel their auction. During the `Open` state, bidders can submit bids, increase their bids, or
    /// cancel their bids if they wish to do so. Also, during this period, the seller is free to cancel the auction at
    /// any point during this period.
    Open,

    /// An auction is said to be settled if the period of the auction has ended and we have successfully been able to
    /// determine a winner of the auction. If the auction had not bids then it is not possible for it to be settled.
    /// When an auction is in this state, the seller can withdraw the payment that they've received from auctioning off
    /// their tokens. The bidders who did not win the bid can withdraw and cancel their bids, and the bidder who won the
    /// bid can no longer withdraw their funds, only their NFTs.
    Settled,

    /// An auction is said to be canceled if the seller decided that they no longer with to sell their NFTs during the
    /// period in which the auction is open. Or, if there are no bids on the auction and therefore it had to be
    /// canceled.
    Canceled,
}
