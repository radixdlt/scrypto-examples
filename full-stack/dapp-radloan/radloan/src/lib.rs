use scrypto::prelude::*;

#[derive(NonFungibleData, ScryptoSbor)]
pub struct LoanDue {
    pub amount_due: Decimal,
}

#[blueprint]
mod radloan {
    struct RadLoan {
        loan_vault: Vault,
        auth_vault: Vault,
        transient_resource_address: ResourceAddress,
    }

    impl RadLoan {
        pub fn instantiate(initial_liquidity: Bucket) -> ComponentAddress {
            let auth_token = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Admin Authority for RadLoan")
                .restrict_withdraw(AccessRule::DenyAll, LOCKED)
                .mint_initial_supply(1);

            let transient_address = ResourceBuilder::new_uuid_non_fungible::<LoanDue>()
                .metadata("name", "Transient promise token for flashloan")
                .mintable(rule!(require(auth_token.resource_address())), LOCKED)
                .burnable(rule!(require(auth_token.resource_address())), LOCKED)
                .restrict_deposit(rule!(deny_all), LOCKED)
                .create_with_no_initial_supply();

            Self {
                loan_vault: Vault::with_bucket(initial_liquidity),
                auth_vault: Vault::with_bucket(auth_token),
                transient_resource_address: transient_address,
            }
            .instantiate()
            .globalize()
        }

        pub fn available_liquidity(&self) -> Decimal {
            self.loan_vault.amount()
        }

        pub fn add_liquidity(&mut self, tokens: Bucket) {
            self.loan_vault.put(tokens)
        }
    }
}
