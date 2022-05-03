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
        /// at once, this HashMap is used to store all of these NFTs with the hashmap key being the resource address of
        /// these NFTs if they are not all of the same _kind_.
        nft_vaults: HashMap<ResourceAddress, Vault>,

        /// Since this is an English Auction, it means that there will be multiple people bidding on the same NFT(s) at
        /// the same time. This hashmaps maps the bidder's badge to a vault which contains the funds that they bid.
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
            }
            .instantiate()
            .add_access_check(access_rules)
            .globalize();

            return (english_auction_sale, ownership_badge);
        }

        /// Cancels the auctioning of tokens and returns them back to their owner.
        ///
        /// This method performs a single check before canceling the auction.
        ///
        /// * **Check 1:** Checks that the auction period has not yet ended.
        ///
        /// # Returns:
        ///
        /// * `Vec<Bucket>` - A vector of buckets of the non-fungible tokens which were being sold.
        ///
        /// # Note:
        ///
        /// * This is an authenticated method which may only be called by the holder of the `ownership_badge`.
        pub fn cancel_auction(&mut self) -> Vec<Bucket> {
            // Checking if the auction can be canceled.
            assert!(
                !self.has_auction_ended(),
                "[Cancel Auction]: Can not cancel an auction after the auction period has passed"
            );

            // At this point we know that the auction can be canceled. So, we withdraw the NFTs and return them to the
            // caller
            let resource_addresses: Vec<ResourceAddress> =
                self.nft_vaults.keys().cloned().collect();
            let mut tokens: Vec<Bucket> = Vec::new();
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
        /// * **Check 1:** Checks that the tokens have already been sold.
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
            // Checking if the payment can be withdrawn
            assert!(
                self.has_auction_ended(),
                "[Withdraw Payment]: Can not withdraw the payment before the auction has ended"
            );
            assert!(
                !self.is_locked(),
                "[Withdraw Payment]: Payment withdrawals are currently disabled. To re-enable it, call the determine_winner method first"
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
        /// * **Check 1:** Checks that the auction has not ended.
        /// * **Check 2:** Checks that the auction has not been canceled.
        /// * **Check 3:** Checks that the payment was provided in the required token.
        ///
        /// # Arguments:
        ///
        /// * `funds` (Bucket) - A bucket of the funds to bid.
        ///
        /// # Returns:
        ///
        /// * `Bucket` - A bucket of the bidder's badge.
        pub fn bid(&mut self, funds: Bucket) -> Bucket {
            // Performing checks to ensure that the bid can be added
            assert!(
                !self.has_auction_ended(),
                "[Bid]: Can not add a bid after the end of the auction"
            );
            assert!(
                !self.is_auction_canceled_or_ended(),
                "[Bid]: The auction has been canceled or ended."
            );
            assert_eq!(
                funds.resource_address(),
                self.accepted_payment_token,
                "[Bid]: Invalid tokens were provided as payment. Payment are only allowed in {}",
                self.accepted_payment_token
            );

            // At this point we know that a bid can be added.

            // Issuing a bidder's NFT to this bidder with information on the amount that they're bidding
            let non_fungible_id: NonFungibleId = NonFungibleId::random();

            let bidders_badge: Bucket = self.internal_admin_badge.authorize(|| {
                let bidders_resource_manager: &ResourceManager =
                    borrow_resource_manager!(self.bidders_badge);
                bidders_resource_manager.mint_non_fungible(
                    &non_fungible_id.clone(),
                    BidderBadge {
                        bid_amount: funds.amount(),
                        is_winner: false,
                    },
                )
            });

            // Taking the bidder's funds and depositing them into a newly created vault where their funds will now live
            self.bid_vaults
                .insert(non_fungible_id, Vault::with_bucket(funds));

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
        /// * **Check 1:** Checks that the auction period has not ended.
        /// * **Check 2:** Checks that the auction has not been canceled.
        /// * **Check 3:** Checks that the payment was provided in the required token.
        /// * **Check 4:** Checks that the badge provided is a valid bidder's badge.
        /// * **Check 5:** Checks that the `Proof` contains a single bidder's badge.
        ///
        /// # Arguments:
        ///
        /// * `funds` (Bucket) - A bucket of the funds the bidder wishes to add to their bid.
        /// * `bidders_badge` (Proof) - A `Proof` of the bidder's badge.
        pub fn increase_bid(&mut self, funds: Bucket, bidders_badge: Proof) {
            // Checking if the bid can be increased or not.
            assert!(
                !self.has_auction_ended(),
                "[Increase Bid]: Can not increase bid after the ending epoch."
            );
            assert!(
                !self.is_auction_canceled_or_ended(),
                "[Increase Bid]: The auction has been canceled or ended."
            );
            assert_eq!(
                funds.resource_address(),
                self.accepted_payment_token,
                "[Increase Bid]: Invalid tokens were provided as bid. Bids are only allowed in {}",
                self.accepted_payment_token
            );

            assert_eq!(
                bidders_badge.resource_address(),
                self.bidders_badge,
                "[Increase Bid]: Badge provided is not a valid bidder's badge"
            );
            assert_eq!(
                bidders_badge.amount(), Decimal::one(),
                "[Increase Bid]: This method requires that exactly one bidder's badge is passed to the method"
            );

            // At this point we know that the bidder's bid can be increased.

            // Updating the metadata of the bidder's badge to reflect on the update of the bidder's bid
            self.internal_admin_badge.authorize(|| {
                let mut bidders_badge_data: BidderBadge = bidders_badge.non_fungible().data();
                bidders_badge_data.bid_amount += funds.amount();
                bidders_badge.non_fungible().update_data(bidders_badge_data);
            });

            // Adding the funds to the vault of the bidder
            self.bid_vaults
                .get_mut(&bidders_badge.non_fungible::<BidderBadge>().id())
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
        /// * **Check 1:** Checks that the component is not locked.
        /// * **Check 2:** Checks that the badge provided is a valid bidder's badge.
        /// * **Check 3:** Checks that the `Proof` contains a single bidder's badge.
        /// * **Check 4:** Checks that the badge provided is not the winner's badge.
        ///
        /// # Arguments:
        ///
        /// * `bidders_badge` (Bucket) - A `Bucket` of the bidder's badge.
        ///
        /// # Returns:
        ///
        /// * `Bucket` - A bucket of the funds owed to the bidder.
        pub fn cancel_bid(&mut self, bidders_badge: Bucket) -> Bucket {
            // Checking if the bid can be canceled or not.
            assert!(
                !self.is_locked(),
                "[Cancel Bid]: Bid cancellation is currently disabled. To re-enable it, call the determine_winner method first"
            );

            assert_eq!(
                bidders_badge.resource_address(),
                self.bidders_badge,
                "[Cancel Bid]: Badge provided is not a valid bidder's badge"
            );
            assert_eq!(
                bidders_badge.amount(), Decimal::one(),
                "[Cancel Bid]: This method requires that exactly one bidder's badge is passed to the method"
            );
            assert!(
                !bidders_badge.non_fungible::<BidderBadge>().data().is_winner,
                "[Cancel Bid]: You can not cancel your bid after winning the auction."
            );

            // At this point we know that the bid cancellation can go on.

            // Take out the bidder's funds from their vault
            let funds: Bucket = self
                .bid_vaults
                .get_mut(&bidders_badge.non_fungible::<BidderBadge>().id())
                .unwrap()
                .take_all();

            // This bidder will no longer need their badge. We can now safely burn the badge.
            self.internal_admin_badge.authorize(|| bidders_badge.burn());

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
        /// * **Check 1:** Checks that the auction has been settled and that the winner has been determined.
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
        pub fn claim_nfts(&mut self, bidders_badge: Bucket) -> Vec<Bucket> {
            // Checking if the NFTs in this component can be claimed
            assert!(
                !self.is_locked(),
                "[Claim NFTs]: NFT claim is currently disabled. To re-enable it, call the determine_winner method first"
            );

            assert_eq!(
                bidders_badge.resource_address(),
                self.bidders_badge,
                "[Claim NFTs]: Badge provided is not a valid bidder's badge"
            );
            assert_eq!(
                bidders_badge.amount(), Decimal::one(),
                "[Claim NFTs]: This method requires that exactly one bidder's badge is passed to the method"
            );
            assert!(
                bidders_badge.non_fungible::<BidderBadge>().data().is_winner,
                "[Claim NFTs]: Badge provided is not the winner's badge."
            );

            // At this point we know that the NFTs can be claimed from the component

            // We can safely burn the bidder's badge at this point as it is no longer needed by the bidder.
            self.internal_admin_badge.authorize(|| bidders_badge.burn());

            // Getting all of the NFTs from the auction and returning them to the caller
            let resource_addresses: Vec<ResourceAddress> =
                self.nft_vaults.keys().cloned().collect();
            let mut tokens: Vec<Bucket> = Vec::new();
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

        // =============================================================================================================
        // The following are general methods which have no restriction in terms of who can and can not call them.
        // =============================================================================================================

        /// Determines the winner of the auction
        ///
        /// This method determines the winner of the auction without returning any of the funds, non-fungible tokens or
        /// anything else. This method is designed to be called by any of the participants once the auction has ended to
        /// determine the winner of the auction. There is no restriction on who can call this (i.e. the seller and
        /// bidders) can both call this method to determine the winner.
        ///
        /// This method first ensures that the auction has ended and that no winner has been selected. Once this check
        /// is done, this method finds the vault which holds the most amount of tokens and selects this vault and its
        /// corresponding `NonFungibleId` as the winner of the bid. From there the badge of the winner is updated to
        /// reflect that and to allow the winner to withdraw their NFTs from the component.
        ///
        /// Once a winner has been determined the component is unlocked and everybody else who had bids can then
        /// withdraw their locked funds back to their accounts.
        ///
        /// This method performs a number of checks before determining the winner:
        ///
        /// * **Check 1:** Checks that the auction has ended.
        /// * **Check 2:** Checks that no winner has been selected so far.
        pub fn determine_winner(&mut self) {
            // Performing the required checks before determining the winner
            assert!(
                self.has_auction_ended(),
                "[Determine Winner]: The auction has not yet ended"
            );
            assert!(
                !self.is_winner_determined(),
                "[Determine Winner]: This auction already has a winner."
            );

            // At this point we can determine the winner of the bid. Here we determine the non-fungible key associated
            // with the vault that holds the most amount of tokens
            let non_fungible_id: NonFungibleId = self
                .bid_vaults
                .iter()
                .max_by(|a, b| a.1.amount().cmp(&b.1.amount()))
                .map(|(k, _v)| k)
                .unwrap()
                .clone();

            // Update the bidder's badge associated with the above non-fungible id to reflect that this is the winner of
            // the bid.
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

            // Take the funds from the winner's vault and put them in the payment vault so that the seller can now
            // withdraw them
            self.payment_vault.put(
                self.bid_vaults
                    .get_mut(&non_fungible_id)
                    .unwrap()
                    .take_all(),
            );
        }

        // =============================================================================================================
        // The following are read-only methods which query the state of the fixed price sale and information about it
        // without performing any state changes. These are useful when connecting a web interface with the component.
        // =============================================================================================================

        /// Returns a boolean of whether the winner has been determined or not.
        ///
        /// This method checks whether the NFTs have been sold or not through the `payment_vault`. If the payment vault
        /// is empty then it means that the tokens have not been sold. On the other hand, if there are funds in the
        /// payment vault then the exchange has gone through and the tokens have been sold.
        ///
        /// # Returns:
        ///
        /// * `bool` - A boolean of whether the bid winner has been determined or not. If this method returns `true`
        /// then the bid winner has been determined. If `false` then the bid winner has not been determined.
        pub fn is_winner_determined(&self) -> bool {
            return !self.payment_vault.is_empty() || self.bid_vaults.values().any(|x| x.amount() == Decimal::zero());
        }

        /// Returns a boolean of whether the auction has ended or not.
        ///
        /// # Returns:
        ///
        /// * `bool` - A boolean of whether the auction has ended or not. If this method returns `true` then the auction
        /// has ended. If it returns `false` then the auction has not ended.
        pub fn has_auction_ended(&self) -> bool {
            return Runtime::current_epoch() >= self.ending_epoch;
        }

        /// Returns a boolean of whether the auction has been settled or not.
        ///
        /// An auction is said to be settled when the auction period has ended and we have selected a winner for the
        /// auction.
        ///
        /// # Returns:
        ///
        /// * `bool` - A boolean of whether the auction has settled or not. If this method returns `true` then the
        /// auction has settled. If `false` then it has not.
        pub fn is_auction_settled(&self) -> bool {
            return self.is_winner_determined() && self.has_auction_ended();
        }

        /// Returns a boolean of whether the auction has been canceled or not.
        ///
        /// This method checks if the auction has been canceled by checking the quantity of non-fungible tokens locked
        /// up in this component. If the quantity is zero then it means that the seller has decided against auctioning
        /// their tokens and has backed out; meaning that the auction is canceled.
        ///
        /// Returns:
        ///
        /// * `bool` - A boolean of whether the auction has been canceled or not. If this method returns `true` then the
        /// auction has been canceled. If `false` then it has not been canceled.
        pub fn is_auction_canceled_or_ended(&self) -> bool {
            return self.non_fungible_addresses().len() == 0;
        }

        /// Returns a boolean of whether the action is locked or unlocked.
        ///
        /// An auction component is locked when the auction period has ended but no winner has been determined yet. When
        /// the component is locked some of the methods and functions may not be called. As an example, when the it is
        /// locked, the `cancel_bid` method may not be called. This is because the auction period has ended and no
        /// winner has been selected. Therefore, we can't allow people to cancel their bids.
        ///
        /// Unlocking the component is rather easy. Anybody can unlock the component by making a call to the method
        /// `determine_winner` on the component. Once a winner is determined the component is unlocked and bidders who
        /// did not win may withdraw their locked up tokens.
        pub fn is_locked(&self) -> bool {
            return if self.is_auction_canceled_or_ended() {
                false
            } else {
                self.has_auction_ended() && !self.is_winner_determined()
            };
        }

        /// Returns a HashMap of the NFTs being auctioned through this component.
        ///
        /// This method returns a `HashMap` of the NFTs being auctioned through this component. The key of the HashMap is the
        /// `ResourceAddress` of the resource and the value is a vector of `NonFungibleIds` belonging to this
        /// `ResourceAddress` that are being auctioned.
        ///
        /// # Returns:
        ///
        /// * `bool` - A HashMap of the non-fungible-ids of the tokens being auctioned.
        pub fn non_fungible_ids(&self) -> HashMap<ResourceAddress, Vec<NonFungibleId>> {
            // Creating the hashmap which we will use to store the resource addresses and the non-fungible-ids.
            let mut mapping: HashMap<ResourceAddress, Vec<NonFungibleId>> = HashMap::new();

            // Adding the entires to the mapping
            let resource_addresses: Vec<ResourceAddress> =
                self.nft_vaults.keys().cloned().collect();
            for resource_address in resource_addresses.into_iter() {
                mapping.insert(
                    resource_address.clone(),
                    self.nft_vaults
                        .get(&resource_address)
                        .unwrap()
                        .non_fungible_ids()
                        .into_iter()
                        .collect::<Vec<NonFungibleId>>(),
                );
            }

            return mapping;
        }

        /// Returns a `NonFungibleAddress` vector of the NFTs being auctioned.
        ///
        /// # Returns:
        ///
        /// * `Vec<NonFungibleAddress>` - A Vector of `NonFungibleAddress`es of the NFTs being auctioned.
        pub fn non_fungible_addresses(&self) -> Vec<NonFungibleAddress> {
            // Creating the vector which will contain the NonFungibleAddresses of the tokens
            let mut vec: Vec<NonFungibleAddress> = Vec::new();

            // Iterate over the items in the hashmap of non-fungible-ids and create the `NonFungibleAddress`es through
            // them
            for (resource_address, non_fungible_ids) in self.non_fungible_ids().iter() {
                vec.append(
                    &mut non_fungible_ids
                        .iter()
                        .map(|x| NonFungibleAddress::new(resource_address.clone(), x.clone()))
                        .collect::<Vec<NonFungibleAddress>>(),
                )
            }

            return vec;
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
