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

        pub fn take_loan(&mut self, loan_amount: Decimal) -> (Bucket, Bucket) {
            assert!(
                loan_amount <= self.loan_vault.amount(),
                "Not enought liquidity to fund loan"
            );

            // Calculate how much must be repaid
            let amount_due = loan_amount * dec!("1.001");

            // Mint an NFT with the loan terms.  Remember that this resource previously had rules defined which
            // forbid it from ever being deposited in any vault.  Thus, once it is present in the transaction
            // the only way for the TX to complete is to remove this "dangling" resource by burning it.
            //
            // Our component will control the only badge with the authority to burn the resource, so anyone taking
            // a loan must call our repay_loan() method with an appropriate reimbursement, at which point we will
            // burn the NFT and allow the TX to complete.
            let loan_terms = self.auth_vault.authorize(|| {
                borrow_resource_manager!(self.transient_resource_address).mint_uuid_non_fungible(
                    LoanDue {
                        amount_due: amount_due,
                    },
                )
            });
            (self.loan_vault.take(loan_amount), loan_terms)
        }

        pub fn repay_loan(&mut self, loan_repayment: Bucket, loan_terms: Bucket) {
            assert!(
                loan_terms.resource_address() == self.transient_resource_address,
                "Incorrect resource passed in for loan terms"
            );

            // Verify we are being sent at least the amount due
            let terms: LoanDue = loan_terms.non_fungible().data();
            assert!(
                loan_repayment.amount() >= terms.amount_due,
                "Insufficient repayment given for your loan!"
            );

            // We could also verify that the resource being repaid is of the correct kind, and give a friendly
            // error message if not. For this example we'll just let the engine handle that when we try to deposit
            self.loan_vault.put(loan_repayment);

            // We have our payment; we can now burn the transient token
            self.auth_vault.authorize(|| loan_terms.burn());
        }
    }
}
