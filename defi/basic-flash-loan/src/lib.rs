use scrypto::prelude::*;

#[derive(NonFungibleData, ScryptoSbor)]
pub struct LoanDue {
    pub amount_due: Decimal,
}

#[blueprint]
mod basic_flash_loan {
    struct BasicFlashLoan {
        loan_vault: Vault,
        transient_resource_manager: ResourceManager,
    }

    impl BasicFlashLoan {
        /// The most elementary possible flash loan.  Creates a loan pool from whatever is initially supplied,
        /// provides loans with a .1% fee, and lets anyone freely add liquidity.
        ///
        /// Does NOT reward liquidity providers in any way or provide a way to remove liquidity from the pool.
        /// Minting LP tokens for rewards, and removing liquidity, is covered in other examples, such as:
        /// https://github.com/radixdlt/scrypto-examples/tree/main/defi/radiswap
        pub fn instantiate_default(initial_liquidity: Bucket) -> Global<BasicFlashLoan> {

            let (address_reservation, component_address) = 
                Runtime::allocate_component_address(BasicFlashLoan::blueprint_id());

            // Define a "transient" resource which can never be deposited once created, only burned
            let transient_token_manager = ResourceBuilder::new_ruid_non_fungible::<LoanDue>(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => 
                        "Promise token for BasicFlashLoan - must be returned to be burned!".to_owned(), locked;
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
                .deposit_roles(deposit_roles!(
                    depositor => rule!(deny_all);
                    depositor_updater => rule!(deny_all);
                ))
                    
                .create_with_no_initial_supply();

            Self {
                loan_vault: Vault::with_bucket(initial_liquidity),
                transient_resource_manager: transient_token_manager,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(address_reservation)
            .globalize()
        }

        pub fn available_liquidity(&self) -> Decimal {
            self.loan_vault.amount()
        }

        pub fn add_liquidity(&mut self, tokens: Bucket) {
            self.loan_vault.put(tokens);
        }

        pub fn take_loan(&mut self, loan_amount: Decimal) -> (Bucket, Bucket) {
            assert!(
                loan_amount <= self.loan_vault.amount(),
                "Not enough liquidity to supply this loan!"
            );

            // Calculate how much we must be repaid
            let amount_due = loan_amount.checked_mul(dec!("1.001"));

            // Mint an NFT with the loan terms.  Remember that this resource previously had rules defined which
            // forbid it from ever being deposited in any vault.  Thus, once it is present in the transaction
            // the only way for the TX to complete is to remove this "dangling" resource by burning it.
            //
            // Our component will control the only badge with the authority to burn the resource, so anyone taking
            // a loan must call our repay_loan() method with an appropriate reimbursement, at which point we will
            // burn the NFT and allow the TX to complete.
            let loan_terms = self.transient_resource_manager
                .mint_ruid_non_fungible(
                        LoanDue {
                            amount_due: amount_due.unwrap(),
                        },
                    );
            (self.loan_vault.take(loan_amount), loan_terms)
        }

        pub fn repay_loan(&mut self, loan_repayment: Bucket, loan_terms: Bucket) {
            assert!(
                loan_terms.resource_address() 
                == self.transient_resource_manager.address(),
                "Incorrect resource passed in for loan terms"
            );

            // Verify we are being sent at least the amount due
            let terms: LoanDue = loan_terms.as_non_fungible().non_fungible().data();
            assert!(
                loan_repayment.amount() >= terms.amount_due,
                "Insufficient repayment given for your loan!"
            );

            // We could also verify that the resource being repaid is of the correct kind, and give a friendly
            // error message if not. For this example we'll just let the engine handle that when we try to deposit
            self.loan_vault.put(loan_repayment);

            // We have our payment; we can now burn the transient token
            loan_terms.burn();
        }
    }
}
