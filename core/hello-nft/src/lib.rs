use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct Ticket {
    pub row: u32,
    pub column: u32,
}

blueprint! {
    struct HelloNft {
        /// A vault that holds all available tickets.
        available_tickets: Vault,
        /// The price for each ticket.
        ticket_price: Decimal,
        /// A vault for collecting payments.
        collected_xrd: Vault,
    }

    impl HelloNft {
        pub fn instantiate_hello_nft(price: Decimal) -> ComponentAddress {
            // Prepare ticket NFT data
            let mut tickets = Vec::new();
            for row in 1..5 {
                for column in 1..5 {
                    tickets.push((NonFungibleId::random(), Ticket { row, column }));
                }
            }

            // Creates a fixed supply of NFTs.
            let ticket_bucket = ResourceBuilder::new_non_fungible()
                .metadata("name", "Ticket")
                .initial_supply(tickets);

            // Instantiate our component
            Self {
                available_tickets: Vault::with_bucket(ticket_bucket),
                ticket_price: price,
                collected_xrd: Vault::new(RADIX_TOKEN),
            }
            .instantiate()
            .globalize()
        }

        pub fn buy_ticket(&mut self, mut payment: Bucket) -> (Bucket, Bucket) {
            // Take our price out of the payment bucket
            self.collected_xrd.put(payment.take(self.ticket_price));

            // Take any ticket
            let ticket = self.available_tickets.take(1);

            // Return the ticket and change
            (ticket, payment)
        }

        pub fn buy_ticket_by_id(&mut self, id: u64, mut payment: Bucket) -> (Bucket, Bucket) {
            // Take our price out of the payment bucket
            self.collected_xrd.put(payment.take(self.ticket_price));

            // Take the specific ticket
            let ticket = self
                .available_tickets
                .take_non_fungible(&NonFungibleId::from_u64(id));

            // Return the ticket and change
            (ticket, payment)
        }

        pub fn available_ticket_ids(&self) -> BTreeSet<NonFungibleId> {
            self.available_tickets.non_fungible_ids()
        }
    }
}
