use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

#[derive(ScryptoSbor)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    MythicRare,
}

#[derive(NonFungibleData, ScryptoSbor)]
pub struct MagicCard {
    color: Color,
    rarity: Rarity,
    #[mutable]
    level: u8,
}

#[blueprint]
mod magic_card_nft {
    struct MagicCardNft {
        /// A vault that holds all our special cards
        special_cards: Vault,
        /// The price for each special card
        special_card_prices: HashMap<NonFungibleLocalId, Decimal>,
        /// The resource address of all random cards
        random_card_resource_manager: ResourceManager,
        /// The price of each random card
        random_card_price: Decimal,
        /// A counter for ID generation
        random_card_id_counter: u64,
        /// A vault that collects all XRD payments
        collected_xrd: Vault,
    }

    impl MagicCardNft {
        pub fn instantiate_component() -> Global<MagicCardNft> {
            // Creates a fixed set of NFTs
            let special_cards_bucket = ResourceBuilder::new_integer_non_fungible(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "Russ' Magic Card Collection".to_owned(), locked;
                    }
                ))
                .mint_initial_supply([
                    (
                        IntegerNonFungibleLocalId::new(1u64),
                        MagicCard {
                            color: Color::Black,
                            rarity: Rarity::MythicRare,
                            level: 3,
                        },
                    ),
                    (
                        IntegerNonFungibleLocalId::new(2u64),
                        MagicCard {
                            color: Color::Green,
                            rarity: Rarity::Rare,
                            level: 5,
                        },
                    ),
                    (
                        IntegerNonFungibleLocalId::new(3u64),
                        MagicCard {
                            color: Color::Red,
                            rarity: Rarity::Uncommon,
                            level: 100,
                        },
                    ),
                ]);

            // Create an NFT resource with mutable supply
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(MagicCardNft::blueprint_id());

            let random_card_resource_manager =
                ResourceBuilder::new_integer_non_fungible::<MagicCard>(OwnerRole::None)
                    .metadata(metadata!(
                        init {
                            "name" => "Random Cards".to_owned(), locked;
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

            // Instantiate our component
            Self {
                special_cards: Vault::with_bucket(special_cards_bucket.into()),
                special_card_prices: HashMap::from([
                    (NonFungibleLocalId::Integer(1u64.into()), 100.into()),
                    (NonFungibleLocalId::Integer(2u64.into()), 200.into()),
                    (NonFungibleLocalId::Integer(3u64.into()), 123.into()),
                ]),
                random_card_resource_manager,
                random_card_price: 50.into(),
                random_card_id_counter: 0,
                collected_xrd: Vault::new(XRD),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(address_reservation)
            .globalize()
        }

        pub fn buy_special_card(
            &mut self,
            key: NonFungibleLocalId,
            mut payment: Bucket,
        ) -> (NonFungibleBucket, Bucket) {
            // Take our price out of the payment bucket
            let price = self.special_card_prices.remove(&key).unwrap();
            self.collected_xrd.put(payment.take(price));

            // Take the requested NFT
            let nft_bucket = self.special_cards.as_non_fungible().take_non_fungible(&key);

            // Return the NFT and change
            (nft_bucket, payment)
        }

        pub fn buy_random_card(&mut self, mut payment: Bucket) -> (Bucket, Bucket) {
            // Take our price out of the payment bucket
            self.collected_xrd.put(payment.take(self.random_card_price));

            // Mint a new card
            let random_seed = 100; // TODO: obtain from oracle
            let new_card = MagicCard {
                color: Self::random_color(random_seed),
                rarity: Self::random_rarity(random_seed),
                level: random_seed as u8 % 8,
            };
            let nft_bucket = self.random_card_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::Integer(self.random_card_id_counter.into()),
                new_card,
            );
            self.random_card_id_counter += 1;

            // Return the NFT and change
            (nft_bucket, payment)
        }

        pub fn upgrade_my_card(&mut self, nft_bucket: Bucket) -> Bucket {
            assert!(
                nft_bucket.amount() == dec!("1"),
                "We can upgrade only one card each time"
            );

            // Get and update the mutable data
            let nft_local_id: NonFungibleLocalId =
                nft_bucket.as_non_fungible().non_fungible_local_id();

            let resource_manager: ResourceManager = self.random_card_resource_manager;

            let mut non_fungible_data: MagicCard =
                nft_bucket.as_non_fungible().non_fungible().data();

            resource_manager.update_non_fungible_data(
                &nft_local_id,
                "level",
                non_fungible_data.level += 1,
            );

            nft_bucket
        }

        pub fn fuse_my_cards(&mut self, nft_bucket: Bucket) -> Bucket {
            assert!(
                nft_bucket.amount() == dec!("2"),
                "You need to pass 2 NFTs for fusion"
            );
            assert!(
                nft_bucket.resource_address() == self.random_card_resource_manager.address(),
                "Only random cards can be fused"
            );

            // Retrieve the NFT data.
            let card1: MagicCard = nft_bucket.as_non_fungible().non_fungibles()[0].data();
            let card2: MagicCard = nft_bucket.as_non_fungible().non_fungibles()[1].data();
            let new_card = Self::fuse_magic_cards(card1, card2);

            // Burn the original cards
            nft_bucket.burn();

            // Mint a new one.
            let new_non_fungible_bucket = self.random_card_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::Integer(self.random_card_id_counter.into()),
                new_card,
            );

            self.random_card_id_counter += 1;

            new_non_fungible_bucket
        }

        fn fuse_magic_cards(card1: MagicCard, card2: MagicCard) -> MagicCard {
            MagicCard {
                color: card1.color,
                rarity: card2.rarity,
                level: card1.level + card2.level,
            }
        }

        fn random_color(seed: u64) -> Color {
            match seed % 5 {
                0 => Color::White,
                1 => Color::Blue,
                2 => Color::Black,
                3 => Color::Red,
                4 => Color::Green,
                _ => panic!(),
            }
        }

        fn random_rarity(seed: u64) -> Rarity {
            match seed % 4 {
                0 => Rarity::Common,
                1 => Rarity::Uncommon,
                2 => Rarity::Rare,
                3 => Rarity::MythicRare,
                _ => panic!(),
            }
        }
    }
}
