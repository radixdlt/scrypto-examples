use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::{builder::ManifestBuilder, prelude::PreviewFlags};

#[test]
fn test_hello() {
    // Setup the environment
    let mut test_runner = TestRunner::builder().build();

    // Create an account
    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());

    // Test the `instantiate_hello` function.
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Hello", "instantiate_hello", manifest_args!())
        .deposit_batch(account_component);

    let manifest_name = manifest.object_names().clone();

    dump_manifest_to_file_system(
        &manifest.build(), // `.build` takes ownership of `Self`.
        manifest_name, // Value moved error from `manifest` here.
        "./transaction_manifest", 
        Some("instantiate_hello"), 
        &NetworkDefinition::simulator()
    ).err();

    // let preview_receipt = test_runner.preview_manifest(
    //     manifest.clone(), 
    //     vec![PublicKey::from(public_key)], 
    //     1, 
    //     PreviewFlags::default());   

    

    // let component = preview_receipt.expect_commit_success().new_component_addresses()[0];

    // let manifest = ManifestBuilder::new()
    //     .call_method(
    //         component, 
    //         "free_token", 
    //         manifest_args!()
    //     )
    //     .deposit_batch(account_component)
    //     .build();

    // let receipt = test_runner.preview_manifest(
    //     manifest.clone(), 
    //     vec![PublicKey::from(public_key)], 
    //     1, 
    //     PreviewFlags::default());  

    // let receipt = test_runner.execute_manifest(
    //     manifest, 
    //     vec![NonFungibleGlobalId::from_public_key(&public_key)]
    // );

    // let execution_receipt = test_runner.execute_manifest(
    //     manifest,
    //     vec![NonFungibleGlobalId::from_public_key(&public_key)],
    // );

    // assert_eq!(
    //     preview_receipt.expect_commit(true).new_component_addresses(),
    //     execution_receipt
    //         .expect_commit(true)
    //         .new_component_addresses()
    // );

    // println!("{}\n", preview_receipt.display(&AddressBech32Encoder::for_simulator()));
    // println!("{}\n", receipt.display(&AddressBech32Encoder::for_simulator()));
    // println!("{}\n", execution_receipt.display(&AddressBech32Encoder::for_simulator()));
    // receipt.expect_commit_success();
    // let component = receipt
    //     .expect_commit(true).new_component_addresses()[0];

}
