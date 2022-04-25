use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_withdraw_all() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let (pk, sk, account) = executor.new_account();
    let package = executor.publish_package(compile_package!()).unwrap();

    println!("Publishing");
    // Publish FlatAdmin
    executor.overwrite_package(
        PackageAddress::from_str("01a99c5f6d0f4b92e81968405bde0e14709ab6630dc0e215a38eef").unwrap(),
        compile_package!("../flat-admin"),
    );

    // Test the `instantiate_managed_access` function.
    let transaction1 = TransactionBuilder::new()
        .call_function(package, "ManagedAccess", "instantiate_managed_access", vec![])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt1 = executor.validate_and_execute(&transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    // Test the `withdraw_all` method.
    let managed_access = receipt1.new_component_addresses[1];
    let admin_badge = receipt1.new_resource_addresses[1];
    let transaction2 = TransactionBuilder::new()
        .create_proof_from_account_by_amount(dec!("1"), admin_badge, account)
        .call_method(
            managed_access,
            "withdraw_all",
            args![],
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());
}
