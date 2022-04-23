use sbor::*;
use scrypto::prelude::*;

#[derive(TypeId, Encode, Decode, Describe, Eq, PartialEq)]
pub enum Section {
    Field,
    Luxury,
}

#[derive(TypeId, Encode, Decode, Describe)]
pub enum Team {
    Home,
    Away,
}

#[derive(NonFungibleData)]
pub struct Ticket {
    /// Which seating section is this ticket for
    section: Section,
    /// If the ticket is for the Luxury section, it will have a specific seat
    seat: Option<String>,
    /// Which team did the buyer predict would win
    #[scrypto(mutable)]
    prediction: Team,
}

blueprint! {
    struct SportingEvent {
        tickets: Vault,
        collected_xrd: Vault,
        price_field: Decimal,
        price_luxury: Decimal,
        admin_authority: Vault,
    }

    impl SportingEvent {
        pub fn instantiate_sporting_event() -> ComponentAddress {
            // For simplicity's sake, we will just use all fixed values for our numbers of tickets and their prices, though all of those could be parameterized

            // We'll start by creating our admin badge which is able to create and modify our NFT
            let my_admin = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);

            // Putting the admin badge in the component auth zone as it will be used throughout this function multiple
            // times. After we're done using it, we will take it back and drop the proof
            ComponentAuthZone::push(my_admin.create_proof());

            // Create our NFT
            let my_non_fungible_address = ResourceBuilder::new_non_fungible()
                .metadata("name", "Ticket to the big game")
                .mintable(rule!(require(my_admin.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(my_admin.resource_address())), LOCKED)
                .no_initial_supply();

            // Currently, Scrypto requires manual assignment of NFT IDs
            let mut ticket_bucket = Bucket::new(my_non_fungible_address);
            let ticket_resource_manager = borrow_resource_manager!(ticket_bucket.resource_address());
            let mut manual_id = 1u64;

            // Mint the Luxury seat tokens.  These seats have an assigned seat number
            // We will default to a prediction of the Home team winning, and purchasers may alter this when they buy their ticket
            for letter in 'A'..'D' {
                for number in 1..10 {
                    let ticket = Ticket {
                        section: Section::Luxury,
                        seat: Some(format!("{}{}", letter, number)),
                        prediction: Team::Home,
                    };
                    ticket_bucket.put(
                        ticket_resource_manager.mint_non_fungible(&NonFungibleId::from_u64(manual_id), ticket)
                    );
                    manual_id += 1;
                }
            }

            // Mint the Field level seats.  These are common seating, with no seat number.  As with Luxury, they will default to a Home win prediction
            // While these tokens each will have unique IDs, they will be otherwise identical
            for manual_id in 101u64..200u64 {
                let ticket = Ticket {
                    section: Section::Field,
                    seat: None,
                    prediction: Team::Home,
                };
                ticket_bucket.put(
                    ticket_resource_manager.mint_non_fungible(&NonFungibleId::from_u64(manual_id), ticket)
                );
            }

            // Dropping the my admin proof
            ComponentAuthZone::pop().drop();

            // Instantiate our component with our supply of sellable tickets
            Self {
                tickets: Vault::with_bucket(ticket_bucket),
                collected_xrd: Vault::new(RADIX_TOKEN),
                price_field: 10.into(),
                price_luxury: 100.into(),
                admin_authority: Vault::with_bucket(my_admin),
            }
            .instantiate()
            .globalize()
        }

        /// Helper function to look for a matching ticket
        fn get_ticket(&mut self, section: Section, seat: Option<String>) -> Bucket {
            let nfts = self.tickets.non_fungibles::<Ticket>();
            // Currently, there is no way to search for particular NFT characteristics within a bucket/vault other than iterating through all of them.
            // A better implementation of this simple use case would be to provide a way to map Luxury seat numbers to an ID deterministically,
            // and likely keep them in a separate vault from the Field tokens so that the semi-fungible Field tokens can be immediately grabbed.            
            // This naive implementation is chosen to show the most basic way to achieve the goal.
            for nft in &nfts {
                let ticket: Ticket = nft.data();
                if ticket.section == section && ticket.seat == seat {
                    return self.tickets.take_non_fungible(&nft.id());
                }                
            };

            panic!("Could not find an appropriate ticket!");
        }

        /// Passing an NFT into this function will switch it from the default Home team prediction to an Away team prediction
        fn switch_nft_prediction(&mut self, mut nft_bucket: Bucket) -> Bucket {
            // First, get the current data and change it to the desired state locally
            let mut nft_data: Ticket = nft_bucket.non_fungible().data();
            nft_data.prediction = Team::Away;

            // Then commit our updated data to our NFT
            self.admin_authority.authorize(|| nft_bucket.non_fungible().update_data(nft_data));

            // All done, send it back
            nft_bucket
        }

        /// Purchases a Field level ticket, switching the prediction if appropriate, and returns it along with any change
        pub fn buy_field_ticket(&mut self, will_home_team_win: bool, mut payment: Bucket) -> (Bucket, Bucket) {
            self.collected_xrd.put(payment.take(self.price_field));
            let nft_bucket = self.get_ticket(Section::Field, None);
            if !will_home_team_win {
                return (self.switch_nft_prediction(nft_bucket), payment);
            }
            else {
                return (nft_bucket, payment);
            }
        }

        /// Purchases a Luxury ticket with a specific desired seat, switching the prediction if appropriate, and returns it along with any change
        pub fn buy_luxury_ticket(&mut self, seat: String, will_home_team_win: bool, mut payment: Bucket) -> (Bucket, Bucket) {
            self.collected_xrd.put(payment.take(self.price_luxury));
            let nft_bucket = self.get_ticket(Section::Luxury, Some(seat));
            if !will_home_team_win {
                return (self.switch_nft_prediction(nft_bucket), payment);
            }
            else {
                return (nft_bucket, payment);
            }
        }
    }
}
