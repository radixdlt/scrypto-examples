// TODO: Update Tests
use scrypto::api::node_modules::metadata::{MetadataEntry, MetadataValue};
use scrypto::blueprints::account::ACCOUNT_WITHDRAW_IDENT;
use scrypto::blueprints::account::{AccountWithdrawInput, ACCOUNT_DEPOSIT_BATCH_IDENT};
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use std::path::Path;

external_blueprint! {
    FlatAdminPackageTarget {
      fn instantiate_flat_admin(badge_name: String) -> (ComponentAddress, Bucket);
    }
  }

#[test]
fn test_managed_access() {
    
    let mut test_runner = TestRunner::builder().build();

    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();

    let package_address = test_runner.compile_and_publish("../../flat-admin/");

    println!("{:?}/n", package_address);
    info!("{:?}", package_address);
    


    let manifest = ManifestBuilder::new()
        .call_function(
            package_address, 
            "FlatAdmin", 
            "instantiate_flat_admin",
            manifest_args!("admin_badge") 
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let flat_admin_package_address = receipt.expect_commit_success().new_package_addresses()[0];

    println!("{:?}/n", receipt);



}