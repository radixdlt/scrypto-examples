use scrypto::crypto::{EcdsaPrivateKey, EcdsaPublicKey};
use scrypto::prelude::*;
use scrypto::values::*;

use radix_engine::errors::RuntimeError;
use radix_engine::ledger::*;
use radix_engine::model::{Receipt, SignedTransaction};
use radix_engine::transaction::*;

mod environment;
mod utils;
use crate::environment::*;
use crate::utils::*;

pub static BLUEPRINT_NAME: &str = "EnglishAuction";
pub static INSTANTIATION_FUNCTION_NAME: &str = "instantiate_english_auction";
pub static BIDDING_PERIOD: u64 = 50u64;

// There are three main actors used throughout the tests you see here:
// * Admin Account: This is the instantiator of the component, i.e. the seller of the tokens
// * Account[0]: This is the account that submitted the losing bid (lower bid).
// * Account[1]: This is the account that submitted the winning bid (higher bid).

// =====================================================================================================================
// The following are stateful tests which test the behavior of the code at different states to ensure that the blueprint
// is behaving correctly throughout it's entire lifetime and under all states.
// =====================================================================================================================

#[test]
pub fn state_setup_succeeds() {
    let funcs: Vec<
        &dyn Fn(
            &mut Environment,
        ) -> (
            ComponentAddress,
            ResourceAddress,
            ResourceAddress,
            ResourceAddress,
        ),
    > = vec![
        &setup_open_state,
        &setup_settled_state,
        &setup_willingly_canceled_state,
    ];

    for func in funcs.iter() {
        let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
        let mut env: Environment = Environment::new(&mut ledger, 10);
        func(&mut env);
    }
}

#[test]
pub fn test_auction_cancellation() {
    // Defining the tests to perform along with the states that hey each correspond to.
    let tests: Vec<(
        &str, // This is the name of the state being tested.
        &dyn Fn(
            &mut Environment,
        ) -> (
            ComponentAddress,
            ResourceAddress,
            ResourceAddress,
            ResourceAddress,
        ), // This is the state function
        bool, // This is a boolean of whether this test should succeed or fail
    )> = vec![
        ("open", &setup_open_state, true),
        ("canceled", &setup_willingly_canceled_state, true),
        ("settled", &setup_settled_state, false),
    ];

    for (state_name, state_func, expected_result) in tests.iter() {
        // Setting up the state for this test
        let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
        let mut env: Environment = Environment::new(&mut ledger, 10);
        let (component_address, ownership_badge, _internal_admin, _bidders_badge): (
            ComponentAddress,
            ResourceAddress,
            ResourceAddress,
            ResourceAddress,
        ) = state_func(&mut env);

        // Performing the actual transaction for the test.
        let transaction: SignedTransaction = TransactionBuilder::new()
            .create_proof_from_account(ownership_badge, env.admin_account.component_address)
            .call_method(component_address, "cancel_auction", args![])
            .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.admin_account.public_key]))
            .sign([&env.admin_account.private_key]);
        let receipt: Receipt = env.executor.validate_and_execute(&transaction).unwrap();

        // Checking that the behavior is as expected.
        let expected_result_string: &str = if expected_result.clone() {
            "succeed"
        } else {
            "fail"
        };
        let assertion_error: String = format!(
            "Expected \"{}\" state test to {} but it did not",
            state_name, expected_result_string
        );

        if expected_result.clone() {
            receipt.result.expect(assertion_error.as_str());
        } else {
            receipt.result.expect_err(assertion_error.as_str());
        };
    }
}

#[test]
pub fn test_payment_withdrawal() {
    // Defining the tests to perform along with the states that hey each correspond to.
    let tests: Vec<(
        &str, // This is the name of the state being tested.
        &dyn Fn(
            &mut Environment,
        ) -> (
            ComponentAddress,
            ResourceAddress,
            ResourceAddress,
            ResourceAddress,
        ), // This is the state function
        bool, // This is a boolean of whether this test should succeed or fail
    )> = vec![
        ("open", &setup_open_state, false),
        ("canceled", &setup_willingly_canceled_state, false),
        ("settled", &setup_settled_state, true),
    ];

    for (state_name, state_func, expected_result) in tests.iter() {
        // Setting up the state for this test
        let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
        let mut env: Environment = Environment::new(&mut ledger, 10);
        let (component_address, ownership_badge, _internal_admin, _bidders_badge): (
            ComponentAddress,
            ResourceAddress,
            ResourceAddress,
            ResourceAddress,
        ) = state_func(&mut env);

        // Performing the actual transaction for the test.
        let transaction: SignedTransaction = TransactionBuilder::new()
            .create_proof_from_account(ownership_badge, env.admin_account.component_address)
            .call_method(component_address, "withdraw_payment", args![])
            .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.admin_account.public_key]))
            .sign([&env.admin_account.private_key]);
        let receipt: Receipt = env.executor.validate_and_execute(&transaction).unwrap();

        // Checking that the behavior is as expected.
        let expected_result_string: &str = if expected_result.clone() {
            "succeed"
        } else {
            "fail"
        };
        let assertion_error: String = format!(
            "Expected \"{}\" state test to {} but it did not",
            state_name, expected_result_string
        );

        if expected_result.clone() {
            receipt.result.expect(assertion_error.as_str());
        } else {
            receipt.result.expect_err(assertion_error.as_str());
        };
    }
}

#[test]
pub fn test_bidding() {
    // Defining the tests to perform along with the states that hey each correspond to.
    let tests: Vec<(
        &str, // This is the name of the state being tested.
        &dyn Fn(
            &mut Environment,
        ) -> (
            ComponentAddress,
            ResourceAddress,
            ResourceAddress,
            ResourceAddress,
        ), // This is the state function
        bool, // This is a boolean of whether this test should succeed or fail
    )> = vec![
        ("open", &setup_open_state, true),
        ("canceled", &setup_willingly_canceled_state, false),
        ("settled", &setup_settled_state, false),
    ];

    for (state_name, state_func, expected_result) in tests.iter() {
        // Setting up the state for this test
        let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
        let mut env: Environment = Environment::new(&mut ledger, 10);
        let (component_address, ownership_badge, _internal_admin, _bidders_badge): (
            ComponentAddress,
            ResourceAddress,
            ResourceAddress,
            ResourceAddress,
        ) = state_func(&mut env);

        // Performing the actual transaction for the test.
        let transaction: SignedTransaction = TransactionBuilder::new()
            .withdraw_from_account_by_amount(
                dec!("1000"),
                RADIX_TOKEN,
                env.accounts[3].component_address,
            )
            .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                builder.call_method(
                    component_address,
                    "bid",
                    args![scrypto::resource::Bucket(bucket_id)],
                )
            })
            .call_method_with_all_resources(env.accounts[3].component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.accounts[3].public_key]))
            .sign([&env.accounts[3].private_key]);
        let receipt: Receipt = env.executor.validate_and_execute(&transaction).unwrap();

        // Checking that the behavior is as expected.
        let expected_result_string: &str = if expected_result.clone() {
            "succeed"
        } else {
            "fail"
        };
        let assertion_error: String = format!(
            "Expected \"{}\" state test to {} but it did not",
            state_name, expected_result_string
        );

        if expected_result.clone() {
            receipt.result.expect(assertion_error.as_str());
        } else {
            receipt.result.expect_err(assertion_error.as_str());
        };
    }
}

#[test]
pub fn test_increase_bid() {
    // Defining the tests to perform along with the states that hey each correspond to.
    let tests: Vec<(
        &str, // This is the name of the state being tested.
        &dyn Fn(
            &mut Environment,
        ) -> (
            ComponentAddress,
            ResourceAddress,
            ResourceAddress,
            ResourceAddress,
        ), // This is the state function
        bool, // This is a boolean of whether this test should succeed or fail
    )> = vec![
        ("open", &setup_open_state, true),
        ("canceled", &setup_willingly_canceled_state, false),
        ("settled", &setup_settled_state, false),
    ];

    for (state_name, state_func, expected_result) in tests.iter() {
        // Setting up the state for this test
        let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
        let mut env: Environment = Environment::new(&mut ledger, 10);
        let (component_address, ownership_badge, _internal_admin, bidders_badge): (
            ComponentAddress,
            ResourceAddress,
            ResourceAddress,
            ResourceAddress,
        ) = state_func(&mut env);

        // Performing the actual transaction for the test.
        let transaction: SignedTransaction = TransactionBuilder::new()
            .withdraw_from_account_by_amount(
                dec!("1000"),
                RADIX_TOKEN,
                env.accounts[5].component_address,
            )
            .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                builder.call_method(
                    component_address,
                    "bid",
                    args![scrypto::resource::Bucket(bucket_id)],
                )
            })
            .call_method_with_all_resources(env.accounts[5].component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.accounts[5].public_key]))
            .sign([&env.accounts[5].private_key]);
        let receipt: Receipt = env.executor.validate_and_execute(&transaction).unwrap();

        let transaction: SignedTransaction = TransactionBuilder::new()
            .create_proof_from_account(bidders_badge, env.accounts[5].component_address)
            .withdraw_from_account_by_amount(
                dec!("1000"),
                RADIX_TOKEN,
                env.accounts[5].component_address,
            )
            .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                builder.create_proof_from_auth_zone(bidders_badge, |builder, proof_id| {
                    builder.call_method(
                        component_address,
                        "increase_bid",
                        args![
                            scrypto::resource::Bucket(bucket_id),
                            scrypto::resource::Proof(proof_id)
                        ],
                    )
                })
            })
            .call_method_with_all_resources(env.accounts[5].component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.accounts[5].public_key]))
            .sign([&env.accounts[5].private_key]);
        let receipt: Receipt = env.executor.validate_and_execute(&transaction).unwrap();

        // Checking that the behavior is as expected.
        let expected_result_string: &str = if expected_result.clone() {
            "succeed"
        } else {
            "fail"
        };
        let assertion_error: String = format!(
            "Expected \"{}\" state test to {} but it did not",
            state_name, expected_result_string
        );

        if expected_result.clone() {
            receipt.result.expect(assertion_error.as_str());
        } else {
            receipt.result.expect_err(assertion_error.as_str());
        };
    }
}

#[test]
pub fn test_non_winner_cancel_bid() {
    // Defining the tests to perform along with the states that hey each correspond to.
    let tests: Vec<(
        &str, // This is the name of the state being tested.
        &dyn Fn(
            &mut Environment,
        ) -> (
            ComponentAddress,
            ResourceAddress,
            ResourceAddress,
            ResourceAddress,
        ), // This is the state function
        bool, // This is a boolean of whether this test should succeed or fail
    )> = vec![
        ("open", &setup_open_state, true),
        ("canceled", &setup_willingly_canceled_state, true),
        ("settled", &setup_settled_state, true),
    ];

    for (state_name, state_func, expected_result) in tests.iter() {
        // Setting up the state for this test
        let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
        let mut env: Environment = Environment::new(&mut ledger, 10);
        let (component_address, _ownership_badge, _internal_admin, bidders_badge): (
            ComponentAddress,
            ResourceAddress,
            ResourceAddress,
            ResourceAddress,
        ) = state_func(&mut env);

        // Performing the actual transaction for the test.
        let transaction: SignedTransaction = TransactionBuilder::new()
            .withdraw_from_account_by_amount(
                dec!("1"),
                bidders_badge,
                env.accounts[0].component_address,
            )
            .take_from_worktop(bidders_badge, |builder, bucket_id| {
                builder.call_method(
                    component_address,
                    "cancel_bid",
                    args![scrypto::resource::Bucket(bucket_id)],
                )
            })
            .call_method_with_all_resources(env.accounts[0].component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.accounts[0].public_key]))
            .sign([&env.accounts[0].private_key]);
        let receipt: Receipt = env.executor.validate_and_execute(&transaction).unwrap();
        println!("At state {}, rec: {:?}", state_name, receipt);

        // Checking that the behavior is as expected.
        let expected_result_string: &str = if expected_result.clone() {
            "succeed"
        } else {
            "fail"
        };
        let assertion_error: String = format!(
            "Expected \"{}\" state test to {} but it did not",
            state_name, expected_result_string
        );

        if expected_result.clone() {
            receipt.result.expect(assertion_error.as_str());
        } else {
            receipt.result.expect_err(assertion_error.as_str());
        };
    }
}

#[test]
pub fn test_winner_cancel_bid() {
    // Defining the tests to perform along with the states that hey each correspond to.
    let tests: Vec<(
        &str, // This is the name of the state being tested.
        &dyn Fn(
            &mut Environment,
        ) -> (
            ComponentAddress,
            ResourceAddress,
            ResourceAddress,
            ResourceAddress,
        ), // This is the state function
        bool, // This is a boolean of whether this test should succeed or fail
    )> = vec![
        ("open", &setup_open_state, true),
        ("canceled", &setup_willingly_canceled_state, true),
        ("settled", &setup_settled_state, false),
    ];

    for (state_name, state_func, expected_result) in tests.iter() {
        // Setting up the state for this test
        let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
        let mut env: Environment = Environment::new(&mut ledger, 10);
        let (component_address, _ownership_badge, _internal_admin, bidders_badge): (
            ComponentAddress,
            ResourceAddress,
            ResourceAddress,
            ResourceAddress,
        ) = state_func(&mut env);

        // Performing the actual transaction for the test.
        let transaction: SignedTransaction = TransactionBuilder::new()
            .withdraw_from_account_by_amount(
                dec!("1"),
                bidders_badge,
                env.accounts[1].component_address,
            )
            .take_from_worktop(bidders_badge, |builder, bucket_id| {
                builder.call_method(
                    component_address,
                    "cancel_bid",
                    args![scrypto::resource::Bucket(bucket_id)],
                )
            })
            .call_method_with_all_resources(env.accounts[1].component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.accounts[1].public_key]))
            .sign([&env.accounts[1].private_key]);
        let receipt: Receipt = env.executor.validate_and_execute(&transaction).unwrap();
        println!("At state {}, rec: {:?}", state_name, receipt);

        // Checking that the behavior is as expected.
        let expected_result_string: &str = if expected_result.clone() {
            "succeed"
        } else {
            "fail"
        };
        let assertion_error: String = format!(
            "Expected \"{}\" state test to {} but it did not",
            state_name, expected_result_string
        );

        if expected_result.clone() {
            receipt.result.expect(assertion_error.as_str());
        } else {
            receipt.result.expect_err(assertion_error.as_str());
        };
    }
}

#[test]
pub fn test_bidder_claim_nft() {
    // Defining the tests to perform along with the states that hey each correspond to.
    let tests: Vec<(
        &str, // This is the name of the state being tested.
        &dyn Fn(
            &mut Environment,
        ) -> (
            ComponentAddress,
            ResourceAddress,
            ResourceAddress,
            ResourceAddress,
        ), // This is the state function
        bool, // This is a boolean of whether this test should succeed or fail
    )> = vec![
        ("open", &setup_open_state, false),
        ("canceled", &setup_willingly_canceled_state, false),
        ("settled", &setup_settled_state, true),
    ];

    for (state_name, state_func, expected_result) in tests.iter() {
        // Setting up the state for this test
        let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
        let mut env: Environment = Environment::new(&mut ledger, 10);
        let (component_address, _ownership_badge, _internal_admin, bidders_badge): (
            ComponentAddress,
            ResourceAddress,
            ResourceAddress,
            ResourceAddress,
        ) = state_func(&mut env);

        // Performing the actual transaction for the test.
        let transaction: SignedTransaction = TransactionBuilder::new()
            .withdraw_from_account_by_amount(
                dec!("1"),
                bidders_badge,
                env.accounts[1].component_address,
            )
            .take_from_worktop(bidders_badge, |builder, bucket_id| {
                builder.call_method(
                    component_address,
                    "claim_nfts",
                    args![scrypto::resource::Bucket(bucket_id)],
                )
            })
            .call_method_with_all_resources(env.accounts[1].component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.accounts[1].public_key]))
            .sign([&env.accounts[1].private_key]);
        let receipt: Receipt = env.executor.validate_and_execute(&transaction).unwrap();
        println!("At state {}, rec: {:?}", state_name, receipt);

        // Checking that the behavior is as expected.
        let expected_result_string: &str = if expected_result.clone() {
            "succeed"
        } else {
            "fail"
        };
        let assertion_error: String = format!(
            "Expected \"{}\" state test to {} but it did not",
            state_name, expected_result_string
        );

        if expected_result.clone() {
            receipt.result.expect(assertion_error.as_str());
        } else {
            receipt.result.expect_err(assertion_error.as_str());
        };
    }
}

// =====================================================================================================================
// The following methods setup the state of the executor to allow for faster tests
// =====================================================================================================================

fn setup_open_state(
    environment: &mut Environment,
) -> (
    ComponentAddress,
    ResourceAddress,
    ResourceAddress,
    ResourceAddress,
) {
    // Creating a new English auction with the 'cars' NFT.
    let component_instantiation_tx: SignedTransaction = TransactionBuilder::new()
        .withdraw_from_account(
            environment.car_resource_address,
            environment.admin_account.component_address,
        )
        .take_from_worktop(environment.car_resource_address, |builder, bucket_id| {
            builder.call_function(
                environment.package_address,
                BLUEPRINT_NAME,
                INSTANTIATION_FUNCTION_NAME,
                args![
                    vec![scrypto::resource::Bucket(bucket_id)],
                    RADIX_TOKEN,
                    BIDDING_PERIOD
                ],
            )
        })
        .call_method_with_all_resources(
            environment.admin_account.component_address,
            "deposit_batch",
        )
        .build(
            environment
                .executor
                .get_nonce([environment.admin_account.public_key]),
        )
        .sign([&environment.admin_account.private_key]);

    let component_instantiation_receipt: Receipt = environment
        .executor
        .validate_and_execute(&component_instantiation_tx)
        .unwrap();
    assert!(component_instantiation_receipt.result.is_ok());

    let component_address: ComponentAddress =
        component_instantiation_receipt.new_component_addresses[0];
    let (
        ownership_badge_resource_address,
        internal_admin_resource_address,
        bidders_badge_resource_address,
    ): (ResourceAddress, ResourceAddress, ResourceAddress) = (
        component_instantiation_receipt.new_resource_addresses[0],
        component_instantiation_receipt.new_resource_addresses[1],
        component_instantiation_receipt.new_resource_addresses[2],
    );

    // Making multiple bids
    let bidding_tx: SignedTransaction = TransactionBuilder::new()
        .withdraw_from_account_by_amount(
            dec!("1000000"),
            RADIX_TOKEN,
            environment.admin_account.component_address,
        )
        .take_from_worktop_by_amount(dec!("100"), RADIX_TOKEN, |builder, bucket_id| {
            builder.call_method(
                component_address,
                "bid",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        // Account 0 has the losing bids.
        .take_from_worktop(bidders_badge_resource_address, |builder, bucket_id| {
            builder.call_method(
                environment.accounts[0].component_address,
                "deposit",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        // Account 1 has the winning bids
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_method(
                component_address,
                "bid",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .call_method_with_all_resources(environment.accounts[1].component_address, "deposit_batch")
        .build(
            environment
                .executor
                .get_nonce([environment.admin_account.public_key]),
        )
        .sign([&environment.admin_account.private_key]);
    let bidding_receipt: Receipt = environment
        .executor
        .validate_and_execute(&bidding_tx)
        .unwrap();
    assert!(bidding_receipt.result.is_ok());

    // Returning everything which is required after creating the state
    return (
        component_address,
        ownership_badge_resource_address,
        internal_admin_resource_address,
        bidders_badge_resource_address,
    );
}

fn setup_settled_state(
    environment: &mut Environment,
) -> (
    ComponentAddress,
    ResourceAddress,
    ResourceAddress,
    ResourceAddress,
) {
    // Using the `setup_open_state` to move setup the open state so we can move forward on the state to the next state.
    let (component_address, ownership_badge, internal_admin, bidders_badge): (
        ComponentAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
    ) = setup_open_state(environment);

    // Advancing the epochs by the `BIDDING_PERIOD`
    environment
        .executor
        .substate_store_mut()
        .set_epoch(BIDDING_PERIOD + 1);

    let settlement_tx: SignedTransaction = TransactionBuilder::new()
        .call_method(component_address, "ensure_auction_settlement", args![])
        .build(
            environment
                .executor
                .get_nonce([environment.admin_account.public_key]),
        )
        .sign([&environment.admin_account.private_key]);
    let settlement_receipt: Receipt = environment
        .executor
        .validate_and_execute(&settlement_tx)
        .unwrap();
    println!("The interesting part is: {:?}", settlement_receipt);
    assert!(settlement_receipt.result.is_ok());

    // At the current moment of time the component should be locked. The addresses may now be returned.
    return (
        component_address,
        ownership_badge,
        internal_admin,
        bidders_badge,
    );
}

fn setup_willingly_canceled_state(
    environment: &mut Environment,
) -> (
    ComponentAddress,
    ResourceAddress,
    ResourceAddress,
    ResourceAddress,
) {
    // Using the `setup_open_state` to move setup the open state so we can move forward on the state to the next state.
    let (component_address, ownership_badge, internal_admin, bidders_badge): (
        ComponentAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
    ) = setup_open_state(environment);

    // Canceling the auction.
    let cancel_auction_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(ownership_badge, environment.admin_account.component_address)
        .call_method(component_address, "cancel_auction", args![])
        .call_method_with_all_resources(environment.accounts[1].component_address, "deposit_batch")
        .build(
            environment
                .executor
                .get_nonce([environment.admin_account.public_key]),
        )
        .sign([&environment.admin_account.private_key]);
    let cancel_auction_receipt: Receipt = environment
        .executor
        .validate_and_execute(&cancel_auction_tx)
        .unwrap();
    assert!(cancel_auction_receipt.result.is_ok());

    // At the current moment of time the component should be locked. The addresses may now be returned.
    return (
        component_address,
        ownership_badge,
        internal_admin,
        bidders_badge,
    );
}
