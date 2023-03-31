use scrypto::prelude::*;

#[derive(NonFungibleData, ScryptoSbor)]
struct StaffBadge {
    employee_name: String,
}

#[blueprint]
mod gumball_machine {
    struct GumballMachine {
        gumballs: Vault,
        collected_xrd: Vault,
        price: Decimal,
        staff_badge_address: ResourceAddress,
    }

    impl GumballMachine {
        // given a price in XRD, creates a ready-to-use gumball machine
        pub fn instantiate_gumball_machine(
            price: Decimal,
            flavor: String,
        ) -> (ComponentAddress, Bucket) {
            let admin_badge: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "admin badge")
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);

            let staff_badge: ResourceAddress =
                ResourceBuilder::new_uuid_non_fungible::<StaffBadge>()
                    .metadata("name", "staff_badge")
                    .mintable(rule!(require(admin_badge.resource_address())), LOCKED)
                    .create_with_no_initial_supply();

            // create a new Gumball resource, with a fixed quantity of 100
            let bucket_of_gumballs = ResourceBuilder::new_fungible()
                .metadata("name", "Gumball")
                .metadata("symbol", flavor)
                .metadata("description", "A delicious gumball")
                .mintable(
                    rule!(require(admin_badge.resource_address()) || require(staff_badge)),
                    LOCKED,
                )
                .mint_initial_supply(100);

            // populate a GumballMachine struct and instantiate a new component
            let component = Self {
                gumballs: Vault::with_bucket(bucket_of_gumballs),
                collected_xrd: Vault::new(RADIX_TOKEN),
                price: price,
                staff_badge_address: staff_badge,
            }
            .instantiate();

            let access_rules = AccessRulesConfig::new()
                .method(
                    "set_price",
                    rule!(require(admin_badge.resource_address())),
                    LOCKED,
                )
                .method(
                    "withdraw_earnings",
                    rule!(require(admin_badge.resource_address())),
                    LOCKED,
                )
                .default(AccessRule::AllowAll, AccessRule::DenyAll);

            (
                component.globalize_with_access_rules(access_rules),
                admin_badge,
            )
        }

        pub fn get_price(&self) -> Decimal {
            self.price
        }

        pub fn set_price(&mut self, price: Decimal) {
            self.price = price
        }

        pub fn mint_staff_badge(&mut self, employee_name: String) -> Bucket {
            let staff_resourcemanager: ResourceManager =
                borrow_resource_manager!(self.staff_badge_address);
            let staff_badge_bucket: Bucket =
                staff_resourcemanager.mint_uuid_non_fungible(StaffBadge {
                    employee_name: employee_name,
                });
            staff_badge_bucket
        }

        pub fn refill_gumball_machine(&mut self) {
            // mint some more gumball tokens requires an admin or staff badge
        }

        pub fn buy_gumball(&mut self, mut payment: Bucket) -> (Bucket, Bucket) {
            // take our price in XRD out of the payment
            // if the caller has sent too few, or sent something other than XRD, they'll get a runtime error
            let our_share = payment.take(self.price);
            self.collected_xrd.put(our_share);

            // we could have simplified the above into a single line, like so:
            // self.collected_xrd.put(payment.take(self.price));

            // return a tuple containing a gumball, plus whatever change is left on the input payment (if any)
            // if we're out of gumballs to give, we'll see a runtime error when we try to grab one
            (self.gumballs.take(1), payment)
        }

        pub fn withdraw_earnings(&mut self) -> Bucket {
            self.collected_xrd.take_all()
        }
    }
}
