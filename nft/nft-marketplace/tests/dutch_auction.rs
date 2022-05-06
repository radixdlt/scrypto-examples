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

pub static BLUEPRINT_NAME: &str = "DutchAuction";
pub static INSTANTIATION_FUNCTION_NAME: &str = "instantiate_dutch_auction_sale";

#[test]
pub fn authenticated_methods_require_badges() {
    // Setting up the environment
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut env: Environment = Environment::new(&mut ledger, 10);

    // Creating the Fixed Price component
    let component_instantiation_tx: SignedTransaction = TransactionBuilder::new()
        .withdraw_from_account(
            env.car_resource_address,
            env.admin_account.component_address,
        )
        .take_from_worktop(env.car_resource_address, |builder, bucket_id| {
            builder.call_function(
                env.package_address,
                BLUEPRINT_NAME,
                INSTANTIATION_FUNCTION_NAME,
                args![
                    vec![scrypto::resource::Bucket(bucket_id)],
                    RADIX_TOKEN,
                    dec!("200"),
                    dec!("100"),
                    50u64
                ],
            )
        })
        .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
        .build(env.executor.get_nonce([env.admin_account.public_key]))
        .sign([&env.admin_account.private_key]);
    let component_instantiation_receipt: Receipt = env
        .executor
        .validate_and_execute(&component_instantiation_tx)
        .unwrap();
    let component_address: ComponentAddress =
        component_instantiation_receipt.new_component_addresses[0];
    let ownership_badge: ResourceAddress =
        component_instantiation_receipt.new_resource_addresses[0];

    // The methods which we would like to perform tests on
    let authenticated_methods: Vec<String> = vec!["cancel_sale", "withdraw_payment"]
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    // Calls to methods should fail with an authorization error when the correct badge is not provided.
    for method_name in authenticated_methods.iter() {
        let method_tx: SignedTransaction = TransactionBuilder::new()
            .call_method(component_address, method_name, args![])
            .build(env.executor.get_nonce([env.admin_account.public_key]))
            .sign([&env.admin_account.private_key]);
        let method_receipt: Receipt = env.executor.validate_and_execute(&method_tx).unwrap();

        let runtime_error: RuntimeError =
            method_receipt.result.expect_err("Transaction should fail");
        assert_auth_error!(runtime_error);
    }

    // Calls to methods should fail with an invoke error when the correct badge is provided (but args are messed up)
    for method_name in authenticated_methods.iter() {
        let method_tx: SignedTransaction = TransactionBuilder::new()
            .create_proof_from_account_by_amount(
                dec!("1"),
                ownership_badge,
                env.admin_account.component_address,
            )
            .call_method(component_address, method_name, args![])
            .build(env.executor.get_nonce([env.admin_account.public_key]))
            .sign([&env.admin_account.private_key]);
        let method_receipt: Receipt = env.executor.validate_and_execute(&method_tx).unwrap();

        let runtime_error: RuntimeError =
            method_receipt.result.expect_err("Transaction should fail");
        if matches!(
            runtime_error,
            RuntimeError::AuthorizationError {
                authorization: _,
                function: _,
                error: ::radix_engine::model::MethodAuthorizationError::NotAuthorized
            }
        ) {
            panic!("Found an unexpected authorization error");
        }
    }
}

#[test]
pub fn full_run_succeeds() {
    // Setting up the environment
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut env: Environment = Environment::new(&mut ledger, 10);

    // Creating the Fixed Price component
    let component_instantiation_tx: SignedTransaction = TransactionBuilder::new()
        .withdraw_from_account(
            env.car_resource_address,
            env.admin_account.component_address,
        )
        .take_from_worktop(env.car_resource_address, |builder, bucket_id| {
            builder.call_function(
                env.package_address,
                BLUEPRINT_NAME,
                INSTANTIATION_FUNCTION_NAME,
                args![
                    vec![scrypto::resource::Bucket(bucket_id)],
                    RADIX_TOKEN,
                    dec!("200"),
                    dec!("100"),
                    50u64
                ],
            )
        })
        .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
        .build(env.executor.get_nonce([env.admin_account.public_key]))
        .sign([&env.admin_account.private_key]);
    let component_instantiation_receipt: Receipt = env
        .executor
        .validate_and_execute(&component_instantiation_tx)
        .unwrap();
    let component_address: ComponentAddress =
        component_instantiation_receipt.new_component_addresses[0];
    let ownership_badge: ResourceAddress =
        component_instantiation_receipt.new_resource_addresses[0];

    // Buying the non-fungible tokens from another account
    env.executor.substate_store_mut().set_epoch(25u64);
    let purchase_tx: SignedTransaction = TransactionBuilder::new()
        .withdraw_from_account_by_amount(
            dec!("150"),
            RADIX_TOKEN,
            env.accounts[0].component_address,
        )
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_method(
                component_address,
                "buy",
                args![scrypto::resource::Bucket(bucket_id)],
            )
        })
        .call_method_with_all_resources(env.accounts[0].component_address, "deposit_batch")
        .build(env.executor.get_nonce([env.accounts[0].public_key]))
        .sign([&env.accounts[0].private_key]);
    let purchase_receipt: Receipt = env.executor.validate_and_execute(&purchase_tx).unwrap();
    assert!(purchase_receipt.result.is_ok(), "Buying NFTs failed");

    // Withdrawing the owed funds from the admin account
    let payment_withdrawal_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(ownership_badge, env.admin_account.component_address)
        .call_method(component_address, "withdraw_payment", args![])
        .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
        .build(env.executor.get_nonce([env.admin_account.public_key]))
        .sign([&env.admin_account.private_key]);
    let payment_withdrawal_receipt: Receipt = env
        .executor
        .validate_and_execute(&payment_withdrawal_tx)
        .unwrap();
    assert!(
        payment_withdrawal_receipt.result.is_ok(),
        "Payment withdrawal has failed."
    )
}
