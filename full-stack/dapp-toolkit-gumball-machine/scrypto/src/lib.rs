use scrypto::prelude::*;

#[derive(NonFungibleData, ScryptoSbor)]
struct StaffBadge {
    employee_name: String,
}

#[blueprint]
mod gumball_machine {
    enable_method_auth! {
        roles {
            admin => updatable_by: [OWNER];
            staff => updatable_by: [admin, OWNER];
        },
        methods {
            buy_gumball => PUBLIC;
            get_price => PUBLIC;
            set_price => restrict_to: [admin, OWNER];
            withdraw_earnings => restrict_to: [OWNER];
            refill_gumball_machine => restrict_to: [staff, admin, OWNER];
            mint_staff_badge => restrict_to: [admin, OWNER];
        }
    }
    struct GumballMachine {
        gumballs: Vault,
        collected_xrd: Vault,
        price: Decimal,
        gum_resource_manager: ResourceManager,
        staff_badge_resource_manager: ResourceManager,
    }

    impl GumballMachine {
        // given a price in XRD, creates a ready-to-use gumball machine
        pub fn instantiate_gumball_machine(
            price: Decimal,
            flavor: String,
        ) -> (Global<GumballMachine>, FungibleBucket, FungibleBucket) {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(GumballMachine::blueprint_id());

            let owner_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .metadata(metadata!(init{"name"=>"owner badge", locked;}))
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);

            let admin_badge = ResourceBuilder::new_fungible(OwnerRole::Updatable(rule!(require(
                owner_badge.resource_address()
            ))))
            .metadata(metadata!(init{"name"=>"admin badge", locked;}))
            .mint_roles(mint_roles! (
                     minter => rule!(require(global_caller(component_address)));
                     minter_updater => OWNER;
            ))
            .divisibility(DIVISIBILITY_NONE)
            .mint_initial_supply(1);

            let staff_badge =
                ResourceBuilder::new_ruid_non_fungible::<StaffBadge>(OwnerRole::Updatable(rule!(
                    require(owner_badge.resource_address())
                        || require(admin_badge.resource_address())
                )))
                .metadata(metadata!(init{"name" => "staff_badge", locked;}))
                .mint_roles(mint_roles! (
                         minter => rule!(require(global_caller(component_address)));
                         minter_updater => OWNER;
                ))
                .burn_roles(burn_roles! (
                    burner => rule!(require(admin_badge.resource_address()));
                    burner_updater => OWNER;
                ))
                .recall_roles(recall_roles! {
                    recaller => rule!(require(admin_badge.resource_address()));
                    recaller_updater => OWNER;
                })
                .create_with_no_initial_supply();

            // create a new Gumball resource, with a fixed quantity of 100
            let bucket_of_gumballs = ResourceBuilder::new_fungible(OwnerRole::None)
                .metadata(metadata!(init{
                    "name" => "Gumball", locked;
                    "symbol" => flavor, locked;
                    "description" => "A delicious gumball", locked;
                }))
                .mint_roles(mint_roles! (
                         minter => rule!(require(global_caller(component_address)));
                         minter_updater => OWNER;
                ))
                .mint_initial_supply(100);

            // populate a GumballMachine struct and instantiate a new component
            let component = Self {
                gum_resource_manager: bucket_of_gumballs.resource_manager(),
                staff_badge_resource_manager: staff_badge,
                gumballs: Vault::with_bucket(bucket_of_gumballs.into()),
                collected_xrd: Vault::new(XRD),
                price: price,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(
                owner_badge.resource_address()
            ))))
            .roles(roles!(
                admin => rule!(require(admin_badge.resource_address()));
                staff => rule!(require(staff_badge.address()));
            ))
            .with_address(address_reservation)
            .globalize();
            return (component, admin_badge, owner_badge);
        }

        pub fn get_price(&self) -> Decimal {
            self.price
        }

        pub fn set_price(&mut self, price: Decimal) {
            self.price = price
        }

        pub fn mint_staff_badge(&mut self, employee_name: String) -> Bucket {
            let staff_badge_bucket: Bucket = self
                .staff_badge_resource_manager
                .mint_ruid_non_fungible(StaffBadge {
                    employee_name: employee_name,
                });
            staff_badge_bucket
        }

        pub fn recall_staff_badge() {
            // recall a staff nft badge and burn it.
        }

        pub fn refill_gumball_machine(&mut self) {
            // mint some more gumball tokens requires an admin or staff badge
            self.gumballs.put(self.gum_resource_manager.mint(100));
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
