use scrypto::api::node_modules::metadata::{MetadataEntry, MetadataValue};
use scrypto::blueprints::account::ACCOUNT_WITHDRAW_IDENT;
use scrypto::blueprints::account::{AccountWithdrawInput, ACCOUNT_DEPOSIT_BATCH_IDENT};
use scrypto::prelude::*;
use scrypto_unit::{Compile, TestRunner, TestRunnerBuilder};
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;

#[test]
fn instantiate() {

let mut test_runner = TestRunner::builder().build();
let (public_key, _private_key, account_component) = test_runner.new_allocated_account();

let package_address = test_runner.compile_and_publish(this_package!());

let instantiate_manifest = ManifestBuilder::new()
    .withdraw_from_account(
        account_component, 
        RADIX_TOKEN, 
        dec!("100")
    )
    .take_from_worktop(
        RADIX_TOKEN, 
        |builder, xrd_bucket| {
            builder.call_function(
                package_address, 
                "BasicFlashLoan", 
                "instantiate_default", 
                manifest_args!(xrd_bucket)
            )
        }
    )
    .call_method(
        account_component, 
        ACCOUNT_DEPOSIT_BATCH_IDENT, 
        manifest_args!(ManifestExpression::EntireWorktop)
    )
    .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        instantiate_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let success = receipt.expect_commit_success();
    let component_address = success.new_component_addresses()[0];
    let transient_token_address = success.new_resource_addresses()[1];

    // Take loan
    let fee = dec!("50") * dec!("1.001");
    let take_loan_manifest = ManifestBuilder::new()
        .call_method(
            component_address, 
            "take_loan", 
            manifest_args!(dec!("50"))
        )
        .withdraw_from_account(
            account_component, 
            RADIX_TOKEN,
            fee)
        .take_from_worktop(
            RADIX_TOKEN, 
            |builder, xrd_bucket| {
            builder.take_from_worktop(transient_token_address, |builder, transient_bucket| {
                builder.call_method(
                    component_address, 
                    "repay_loan", 
                    manifest_args!(xrd_bucket, transient_bucket)
                )
            })
        })
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        take_loan_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    println!("{:?}/n", receipt);

    receipt.expect_commit_success();

}