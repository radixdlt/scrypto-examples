use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_create_additional_admin() {
    // Set up environment.
    let mut test_runner = TestRunnerBuilder::new().build();

    // Create an account
    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());

    // Test the `instantiate_hello_nft` function.
    let manifest1 = ManifestBuilder::new()
        .call_function(
            package_address,
            "HelloNft",
            "instantiate_hello_nft",
            manifest_args!(dec!("5")),
        )
        .call_method(
            account_component,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt1 = test_runner.execute_manifest_ignoring_fee(
        manifest1,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt1);
    receipt1.expect_commit_success();

    // Test the `buy_ticket_by_id` method.
    let component = receipt1.expect_commit(true).new_component_addresses()[0];

    let manifest2 = ManifestBuilder::new()
        .withdraw_from_account(account_component, XRD, dec!("10"))
        .take_all_from_worktop(XRD, "bucket")
        .call_method_with_name_lookup(component, "buy_ticket", |lookup| (lookup.bucket("bucket"),))
        .call_method(
            account_component,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt2 = test_runner.execute_manifest_ignoring_fee(
        manifest2,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt2);
    receipt2.expect_commit_success();
}
