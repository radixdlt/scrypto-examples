use radix_engine::ledger::*;
use scrypto::core::NetworkDefinition;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_create_additional_admin() {
    // Set up environment.
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Create an account
    let (public_key, _private_key, account_component) = test_runner.new_account();

    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());

    // Test the `instantiate_hello_nft` function.
    let manifest1 = ManifestBuilder::new(&NetworkDefinition::simulator())
        .call_function(
            package_address,
            "HelloNft",
            "instantiate_hello_nft",
            args!(dec!("5")),
        )
        .call_method(
            account_component,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt1 = test_runner.execute_manifest_ignoring_fee(manifest1, vec![public_key.into()]);
    println!("{:?}\n", receipt1);
    receipt1.expect_commit_success();

    // Test the `buy_ticket_by_id` method.
    let component = receipt1
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];
    let manifest2 = ManifestBuilder::new(&NetworkDefinition::simulator())
        .withdraw_from_account_by_amount(dec!("10"), RADIX_TOKEN, account_component)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_method(
                component,
                "buy_ticket",
                args!(scrypto::resource::Bucket(bucket_id)),
            )
        })
        .call_method(
            account_component,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt2 = test_runner.execute_manifest_ignoring_fee(manifest2, vec![public_key.into()]);
    println!("{:?}\n", receipt2);
    receipt2.expect_commit_success();
}
