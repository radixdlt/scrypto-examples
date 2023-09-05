use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_magic_card() {
    // Set up environment.
    let mut test_runner = TestRunnerBuilder::new().build();

    // Create an account
    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());

    // Test the `instantiate_component` function.
    let transaction1 = ManifestBuilder::new()
        .call_function(
            package_address,
            "MagicCardNft",
            "instantiate_component",
            manifest_args!(),
        )
        .build();
    let receipt1 = test_runner.execute_manifest_ignoring_fee(
        transaction1,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt1);
    receipt1.expect_commit_success();

    // Test the `buy_special_card` method.
    let component = receipt1.expect_commit(true).new_component_addresses()[0];

    let transaction2 = ManifestBuilder::new()
        .withdraw_from_account(account_component, XRD, dec!("666"))
        .take_all_from_worktop(XRD, "bucket")
        .call_method_with_name_lookup(component, "buy_special_card", |lookup| {
            (NonFungibleLocalId::integer(2u64), lookup.bucket("bucket"))
        })
        .call_method(
            account_component,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt2 = test_runner.execute_manifest_ignoring_fee(
        transaction2,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt2);
    receipt2.expect_commit_success();

    // Test the `buy_special_card` method.
    let component = receipt1.expect_commit(true).new_component_addresses()[0];

    let transaction3 = ManifestBuilder::new()
        .withdraw_from_account(account_component, XRD, dec!("500"))
        .take_all_from_worktop(XRD, "bucket")
        .call_method_with_name_lookup(component, "buy_random_card", |lookup| {
            (lookup.bucket("bucket"),)
        })
        .deposit_batch(account_component)
        .build();
    let receipt3 = test_runner.execute_manifest_ignoring_fee(
        transaction3,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt3);
    receipt3.expect_commit_success();
}
