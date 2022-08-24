use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use scrypto::prelude::*;
use scrypto::args;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_magic_card() {
    // Set up environment.
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Create an account
    let (public_key, _private_key, account_component) = test_runner.new_account();

    // Publish package
    let package_address = test_runner.publish_package(extract_package(compile_package!()).unwrap());

    // Test the `instantiate_component` function.
    let transaction1 = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(package_address, "HelloNft", "instantiate_component", args!())
        .build();
    let receipt1 = test_runner.execute_manifest_ignoring_fee(transaction1, vec![public_key]);
    println!("{:?}\n", receipt1);
    receipt1.expect_success();

    // Test the `buy_special_card` method.
    let component = receipt1.new_component_addresses[0];
    let transaction2 = ManifestBuilder::new(Network::LocalSimulator)
        .withdraw_from_account_by_amount(dec!("666"), RADIX_TOKEN, account_component)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_method(
                component,
                "buy_special_card",
                args!(
                    NonFungibleId::from_u64(2u64),
                    scrypto::resource::Bucket(bucket_id)
                )
            )
        })
        .call_method_with_all_resources(account_component, "deposit_batch")
        .build();
    let receipt2 = test_runner.execute_manifest_ignoring_fee(transaction2, vec![public_key]);
    println!("{:?}\n", receipt2);
    receipt2.expect_success();

    // Test the `buy_special_card` method.
    let component = receipt1.new_component_addresses[0];
    let transaction3 = ManifestBuilder::new(Network::LocalSimulator)
        .withdraw_from_account_by_amount(dec!("1000"), RADIX_TOKEN, account_component)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_method(
                component,
                "buy_random_card",
                args!(
                    scrypto::resource::Bucket(bucket_id)
                )
            )
        })
        .call_method_with_all_resources(account_component, "deposit_batch")
        .build();
    let receipt3 = test_runner.execute_manifest_ignoring_fee(transaction3, vec![public_key]);
    println!("{:?}\n", receipt3);
    receipt3.expect_success();
}
