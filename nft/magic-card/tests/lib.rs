use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_magic_card() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let (pk, sk, account) = executor.new_account();
    let package = executor.publish_package(compile_package!()).unwrap();

    // Test the `instantiate_component` function.
    let transaction1 = TransactionBuilder::new()
        .call_function(package, "HelloNft", "instantiate_component", vec![])
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt1 = executor.validate_and_execute(&transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    // Test the `buy_special_card` method.
    let component = receipt1.new_component_addresses[0];
    let transaction2 = TransactionBuilder::new()
        .withdraw_from_account_by_amount(dec!("666"), RADIX_TOKEN, account)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_method(
                component,
                "buy_special_card",
                to_struct!(
                    NonFungibleId::from_u64(2u64),
                    scrypto::resource::Bucket(bucket_id)
                )
            )
        })
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());

    // Test the `buy_special_card` method.
    let component = receipt1.new_component_addresses[0];
    let transaction3 = TransactionBuilder::new()
        .withdraw_from_account_by_amount(dec!("1000"), RADIX_TOKEN, account)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_method(
                component,
                "buy_random_card",
                to_struct!(
                    scrypto::resource::Bucket(bucket_id)
                )
            )
        })
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt3 = executor.validate_and_execute(&transaction3).unwrap();
    println!("{:?}\n", receipt3);
    assert!(receipt3.result.is_ok());
}
