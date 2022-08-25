use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use scrypto::args;
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
    let package_address = test_runner.publish_package(extract_package(compile_package!()).unwrap());

    // Test the `instantiate_flat_admin` function.
    let manifest1 = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .call_function(
            package_address,
            "FlatAdmin",
            "instantiate_flat_admin",
            args!("test"),
        )
        .call_method_with_all_resources(account_component, "deposit_batch")
        .build();
    let receipt1 = test_runner.execute_manifest_ignoring_fee(manifest1, vec![public_key]);
    println!("{:?}\n", receipt1);
    receipt1.expect_success();

    // Test the `create_additional_admin` method.
    let flat_admin = receipt1
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_component_addresses[0];
    let admin_badge = receipt1
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_resource_addresses[1];
    let manifest2 = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .create_proof_from_account_by_amount(dec!("1"), admin_badge, account_component)
        .call_method(flat_admin, "create_additional_admin", args!())
        .call_method_with_all_resources(account_component, "deposit_batch")
        .build();
    let receipt2 = test_runner.execute_manifest_ignoring_fee(manifest2, vec![public_key]);
    println!("{:?}\n", receipt2);
    receipt2.expect_success();
}
