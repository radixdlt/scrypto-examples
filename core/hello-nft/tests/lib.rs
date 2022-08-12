// use radix_engine::ledger::*;
// use radix_engine::transaction::*;
// use scrypto::prelude::*;

// #[test]
// fn test_hello() {
//     // Set up environment.
//     let mut ledger = InMemorySubstateStore::with_bootstrap();
//     let mut executor = TransactionExecutor::new(&mut ledger, false);
//     let (pk, sk, account) = executor.new_account();
//     let package = executor.publish_package(compile_package!()).unwrap();

//     // Test the `instantiate_hello_nft` function.
//     let transaction1 = TransactionBuilder::new()
//         .call_function(package, "HelloNft", "instantiate_hello_nft", vec![scrypto_encode(&Decimal(5))])
//         .call_method_with_all_resources(account, "deposit_batch")
//         .build(executor.get_nonce([pk]))
//         .sign([&sk]);
//     let receipt1 = executor.validate_and_execute(&transaction1).unwrap();
//     println!("{:?}\n", receipt1);
//     assert!(receipt1.result.is_ok());

//     // Test the `buy_ticket_by_id` method.
//     let component = receipt1.new_component_addresses[0];
//     let transaction2 = TransactionBuilder::new()
//         .withdraw_from_account_by_amount(Decimal(10), RADIX_TOKEN, account)
//         .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
//             builder.call_method(
//                 component,
//                 "buy_ticket",
//                 vec![
//                     scrypto_encode(&scrypto::resource::Bucket(bucket_id)),
//                 ],
//             )
//         })
//         .call_method_with_all_resources(account, "deposit_batch")
//         .build(executor.get_nonce([pk]))
//         .sign([&sk]);
//     let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
//     println!("{:?}\n", receipt2);
//     assert!(receipt2.result.is_ok());
// }

use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto::to_struct;
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
    let package_address = test_runner.publish_package(extract_package(compile_package!()).unwrap());

    // Test the `instantiate_hello_nft` function.
    let manifest1 = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(package_address, "HelloNft", "instantiate_hello_nft", to_struct!(dec!("5")))
        .call_method_with_all_resources(account_component, "deposit_batch")
        .build();
    let receipt1 = test_runner.execute_manifest_ignoring_fee(manifest1, vec![public_key]);
    println!("{:?}\n", receipt1);
    receipt1.expect_success();

    // Test the `buy_ticket_by_id` method.
    let component = receipt1.new_component_addresses[0];
    let manifest2 = ManifestBuilder::new(Network::LocalSimulator)
        .withdraw_from_account_by_amount(dec!("10"), RADIX_TOKEN, account_component)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_method(
                component,
                "buy_ticket",
                to_struct!(scrypto::resource::Bucket(bucket_id))
            )
        })
        .call_method_with_all_resources(account_component, "deposit_batch")
        .build();
    let receipt2 = test_runner.execute_manifest_ignoring_fee(manifest2, vec![public_key]);
    println!("{:?}\n", receipt2);
    receipt2.expect_success();
}