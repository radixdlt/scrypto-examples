use scrypto::api::node_modules::metadata::{MetadataEntry, MetadataValue};
use scrypto::blueprints::account::ACCOUNT_WITHDRAW_IDENT;
use scrypto::blueprints::account::{ACCOUNT_DEPOSIT_BATCH_IDENT};
use scrypto::blueprints::package;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]

fn deploy_radiswap_with_owner() {
    // let mut test_runner = TestRunner::builder().build();

    // let (public_key, _private_key, account_component) = test_runner.new_allocated_account();

    let (code, schema) = Compile::compile(this_package!());

    let package_rules = AccessRulesConfig::new()
        .default(
            rule!(require(RADIX_TOKEN)), 
            rule!(require(RADIX_TOKEN))
        );

        // .insert(
            // "dapp_definition".into(),
        
            // MetadataEntry::List(vec![
            //     MetadataValue::Address(Address::Component(account_component))
            // ])
        // );
    let manifest = ManifestBuilder::new()
        .publish_package(
            code, 
            schema, 
            Default::default(), 
            Default::default(), 
            package_rules
        )
        .build();
        

    utils::write_manifest_to_fs(&manifest, "./7-publishing-packages", 0xf2).unwrap();

}

#[test]
fn set_metadata() {
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();
    
    let manifest = ManifestBuilder::new()
        .create_proof_from_account(
            account_component,
            RADIX_TOKEN
        )
        .set_metadata(
            account_component.into(),
            "metadata_key".into(),
            MetadataEntry::List(vec![
                MetadataValue::Address(Address::Component((account_component))),
            ]),
        )
        .build();
        

    utils::write_manifest_to_fs(&manifest, "./7-publishing-packages", 0xf2).unwrap();

}

#[test]
fn instantiate_radiswap_with_owner() {
    let mut test_runner = TestRunner::builder().build();

    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();

    let package_address = test_runner.compile_and_publish(this_package!());

    // Creating AccessRules for Token B.
    let mut access_rules = BTreeMap::new();
    access_rules.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
    access_rules.insert(ResourceMethodAuthKey::Deposit, (rule!(allow_all), LOCKED));

    // Creating and minting Token B for testing purposes.
    let create_fungible_manifest = ManifestBuilder::new()
        .create_fungible_resource(
            0, 
            Default::default(), 
            access_rules, 
            Some(dec!("1000"))
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop))
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        create_fungible_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    println!("{:?}\n", receipt);

    let token_b_resource_address = receipt.expect_commit_success().new_resource_addresses()[0];

    let swap_fee = dec!("0.02");

    // Instantiating Radiswap with XRD/TokenB pair with 100 supply each pool.
    let instantiate_radiswap_manifest = ManifestBuilder::new()
        .call_method(
            account_component, 
            ACCOUNT_WITHDRAW_IDENT, 
            manifest_args!(RADIX_TOKEN, dec!("100"))
        )
        .call_method(
            account_component, 
            ACCOUNT_WITHDRAW_IDENT, 
            manifest_args!(token_b_resource_address, dec!("100"))
        )
        .take_from_worktop(RADIX_TOKEN, |builder, xrd_bucket| {
            builder.take_from_worktop(token_b_resource_address, |builder, token_b_bucket| {
                builder.call_function(
                    package_address, 
                    "Radiswap", 
                    "instantiate_radiswap", 
                    manifest_args!(
                        xrd_bucket,
                        token_b_bucket,
                        swap_fee // Swap fee @ 2%
                    )
                )
            })
        })
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
        manifest_args!(ManifestExpression::EntireWorktop))
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        instantiate_radiswap_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    println!("{:?}/n", receipt);

    let success = receipt.expect_commit_success();

    let pool_unit_resource_address = success.new_resource_addresses()[1];

    let pool_unit_balance = test_runner.account_balance(
        account_component, 
        pool_unit_resource_address,
    );

    println!("Pool Units: {:?}", pool_unit_balance);

}

#[test]
fn swap() {
    let mut test_runner = TestRunner::builder().build();

    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();

    let package_address = test_runner.compile_and_publish(this_package!());

    // Creating AccessRules for Token B.
    let mut access_rules = BTreeMap::new();
    access_rules.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
    access_rules.insert(ResourceMethodAuthKey::Deposit, (rule!(allow_all), LOCKED));

    // Creating and minting Token B for testing purposes.
    let create_fungible_manifest = ManifestBuilder::new()
        .create_fungible_resource(
            0, 
            Default::default(), 
            access_rules, 
            Some(dec!("1000"))
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop))
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        create_fungible_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    println!("{:?}\n", receipt);

    let token_b_resource_address = receipt.expect_commit_success().new_resource_addresses()[0];

    let swap_fee = dec!("0.02");

    // Instantiating Radiswap with XRD/TokenB pair with 100 supply each pool.
    let instantiate_radiswap_manifest = ManifestBuilder::new()
        .call_method(
            account_component, 
            ACCOUNT_WITHDRAW_IDENT, 
            manifest_args!(RADIX_TOKEN, dec!("100"))
        )
        .call_method(
            account_component, 
            ACCOUNT_WITHDRAW_IDENT, 
            manifest_args!(token_b_resource_address, dec!("100"))
        )
        .take_from_worktop(RADIX_TOKEN, |builder, xrd_bucket| {
            builder.take_from_worktop(token_b_resource_address, |builder, token_b_bucket| {
                builder.call_function(
                    package_address, 
                    "Radiswap", 
                    "instantiate_radiswap", 
                    manifest_args!(
                        xrd_bucket,
                        token_b_bucket,
                        swap_fee // Swap fee @ 2%
                    )
                )
            })
        })
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
        manifest_args!(ManifestExpression::EntireWorktop))
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        instantiate_radiswap_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    println!("{:?}/n", receipt);

    let success = receipt.expect_commit_success();

    let component_address = success.new_component_addresses()[0];
    let pool_unit_resource_address = success.new_resource_addresses()[1];


    // Testing Swap method
    // We will be swapping 10 Token B.
    // To test this, we will create a formula of what amount we expect to receive and 
    // ensure that is what we will actually receive.

    let y_vault_id = test_runner.get_component_vaults(
        component_address, 
        token_b_resource_address
    );
    let y_vault_amount = test_runner.inspect_fungible_vault(y_vault_id[0]);
    let x_vault_id = test_runner.get_component_vaults(
        component_address, 
        RADIX_TOKEN,
    );
    let x_vault_amount = test_runner.inspect_fungible_vault(x_vault_id[0]);
    let dx = dec!(10);
    let r = swap_fee;
    let dy = 
    (y_vault_amount.unwrap() * (dec!("1") - r) * dx) /
     (x_vault_amount.unwrap() + (dec!("1") - r) * dx);


    let swap_manifest = ManifestBuilder::new()
        .call_method(
            account_component,
            ACCOUNT_WITHDRAW_IDENT,
            manifest_args!(token_b_resource_address, dec!("10")),
        )
        .take_from_worktop_by_amount(dec!("10"), token_b_resource_address, |builder, xrd_bucket| {
            builder.call_method(
                component_address, 
                "swap", 
                manifest_args!(xrd_bucket)
            )
        })
        .assert_worktop_contains_by_amount(
            dy, 
            RADIX_TOKEN
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop)
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        swap_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let xrd_balance = test_runner.account_balance(
        account_component, 
        RADIX_TOKEN,
    );

    let token_b_balance = test_runner.account_balance(
        account_component, 
        token_b_resource_address
    );

    println!("DY amount: {:?}", dy);
    println!("XRD balance: {:?}", xrd_balance);
    println!("Token B balance: {:?}", token_b_balance);

    println!("{:?}/n", receipt);
    let success = receipt.expect_commit(true);

    success.balance_changes();
    
}

    
#[test]
fn add_liquidity() {

    let mut test_runner = TestRunner::builder().build();

    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();
    let (public_key2, _private_key2, account_component2) = test_runner.new_allocated_account();

    let package_address = test_runner.compile_and_publish(this_package!());

    // Creating AccessRules for Token B.
    let mut access_rules = BTreeMap::new();
    access_rules.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
    access_rules.insert(ResourceMethodAuthKey::Deposit, (rule!(allow_all), LOCKED));

    // Creating and minting Token B for testing purposes.
    let create_fungible_manifest = ManifestBuilder::new()
        .create_fungible_resource(
            0, 
            Default::default(), 
            access_rules, 
            Some(dec!("1000"))
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop))
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        create_fungible_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    println!("{:?}\n", receipt);

    let token_b_resource_address = receipt.expect_commit_success().new_resource_addresses()[0];

    // Transfering Token B to Account 2
    let token_transfer_manifest = ManifestBuilder::new()
        .withdraw_from_account(
            account_component, 
            token_b_resource_address, 
            dec!("500")
        )
        .call_method(
            account_component2, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        token_transfer_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    println!("{:?}\n", receipt);

    // Instantiating Radiswap with XRD/TokenB pair with 100 supply each pool.
    let swap_fee = dec!("0.02");
    let instantiate_radiswap_manifest = ManifestBuilder::new()
        .call_method(
            account_component, 
            ACCOUNT_WITHDRAW_IDENT, 
            manifest_args!(RADIX_TOKEN, dec!("100"))
        )
        .call_method(
            account_component, 
            ACCOUNT_WITHDRAW_IDENT, 
            manifest_args!(token_b_resource_address, dec!("100"))
        )
        .take_from_worktop(RADIX_TOKEN, |builder, xrd_bucket| {
            builder.take_from_worktop(token_b_resource_address, |builder, token_b_bucket| {
                builder.call_function(
                    package_address, 
                    "Radiswap", 
                    "instantiate_radiswap", 
                    manifest_args!(
                        xrd_bucket,
                        token_b_bucket,
                        swap_fee // Swap fee @ 2%
                    )
                )
            })
        })
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
        manifest_args!(ManifestExpression::EntireWorktop))
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        instantiate_radiswap_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    println!("{:?}/n", receipt);

    let success = receipt.expect_commit_success();
    let component_address = success.new_component_addresses()[0];
    let pool_unit_resource_address = success.new_resource_addresses()[1];


    // Add Liquidity
    // Adding liquidity using Account 2
    let add_liquidity_manifest = ManifestBuilder::new()
        .withdraw_from_account(
            account_component2, 
            RADIX_TOKEN, 
            dec!("100")
        )
        .withdraw_from_account(
            account_component2, 
            token_b_resource_address, 
            dec!("100")
        )
        .take_from_worktop(
            RADIX_TOKEN, 
            |builder, xrd_bucket| {
                builder.take_from_worktop(
                    token_b_resource_address, 
                    |builder, token_b_bucket| {
                        builder.call_method(
                            component_address, 
                            "add_liquidity", 
                            manifest_args!(
                                xrd_bucket,
                                token_b_bucket
                            ))
                    })
            })
        .call_method(
            account_component2, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop)
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        add_liquidity_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key2)],
    );

    println!("{:?}/n", receipt);

    let success = receipt.expect_commit_success();
    success.balance_changes();

    let pool_unit_balance = test_runner.account_balance(
        account_component, 
        pool_unit_resource_address,
    );

    let pool_unit_balance2 = test_runner.account_balance(
        account_component2, 
        pool_unit_resource_address,
    );

    println!("Pool Units: {:?}", pool_unit_balance);
    println!("Pool Units2: {:?}", pool_unit_balance2);
}

#[test]
fn remove_liquidity() {

    let mut test_runner = TestRunner::builder().build();

    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();
    let (public_key2, _private_key2, account_component2) = test_runner.new_allocated_account();

    let package_address = test_runner.compile_and_publish(this_package!());

    // Creating AccessRules for Token B.
    let mut access_rules = BTreeMap::new();
    access_rules.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
    access_rules.insert(ResourceMethodAuthKey::Deposit, (rule!(allow_all), LOCKED));

    // Creating and minting Token B for testing purposes.
    let create_fungible_manifest = ManifestBuilder::new()
        .create_fungible_resource(
            0, 
            Default::default(), 
            access_rules, 
            Some(dec!("1000"))
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop))
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        create_fungible_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    println!("{:?}\n", receipt);

    let token_b_resource_address = receipt.expect_commit_success().new_resource_addresses()[0];

    // Transfering Token B to Account 2
    let token_transfer_manifest = ManifestBuilder::new()
        .withdraw_from_account(
            account_component, 
            token_b_resource_address, 
            dec!("500")
        )
        .call_method(
            account_component2, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        token_transfer_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    println!("{:?}\n", receipt);

    // Instantiating Radiswap with XRD/TokenB pair with 100 supply each pool.
    let swap_fee = dec!("0.02");
    let instantiate_radiswap_manifest = ManifestBuilder::new()
        .call_method(
            account_component, 
            ACCOUNT_WITHDRAW_IDENT, 
            manifest_args!(RADIX_TOKEN, dec!("100"))
        )
        .call_method(
            account_component, 
            ACCOUNT_WITHDRAW_IDENT, 
            manifest_args!(token_b_resource_address, dec!("100"))
        )
        .take_from_worktop(RADIX_TOKEN, |builder, xrd_bucket| {
            builder.take_from_worktop(token_b_resource_address, |builder, token_b_bucket| {
                builder.call_function(
                    package_address, 
                    "Radiswap", 
                    "instantiate_radiswap", 
                    manifest_args!(
                        xrd_bucket,
                        token_b_bucket,
                        swap_fee // Swap fee @ 2%
                    )
                )
            })
        })
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
        manifest_args!(ManifestExpression::EntireWorktop))
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        instantiate_radiswap_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    println!("{:?}/n", receipt);

    let success = receipt.expect_commit_success();
    let component_address = success.new_component_addresses()[0];
    let pool_unit_resource_address = success.new_resource_addresses()[1];


    // Add Liquidity
    // Adding liquidity using Account 2
    let add_liquidity_manifest = ManifestBuilder::new()
        .withdraw_from_account(
            account_component2, 
            RADIX_TOKEN, 
            dec!("100")
        )
        .withdraw_from_account(
            account_component2, 
            token_b_resource_address, 
            dec!("100")
        )
        .take_from_worktop(
            RADIX_TOKEN, 
            |builder, xrd_bucket| {
                builder.take_from_worktop(
                    token_b_resource_address, 
                    |builder, token_b_bucket| {
                        builder.call_method(
                            component_address, 
                            "add_liquidity", 
                            manifest_args!(
                                xrd_bucket,
                                token_b_bucket
                            ))
                    })
            })
        .assert_worktop_contains_by_amount(
            dec!("100"), 
            pool_unit_resource_address
        )
        .call_method(
            account_component2, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop)
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        add_liquidity_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key2)],
    );

    println!("{:?}/n", receipt);

    let success = receipt.expect_commit_success();
    success.balance_changes();

    let pool_unit_balance = test_runner.account_balance(
        account_component, 
        pool_unit_resource_address,
    );

    let pool_unit_balance2 = test_runner.account_balance(
        account_component2, 
        pool_unit_resource_address,
    );

    println!("Pool Units: {:?}", pool_unit_balance);
    println!("Pool Units2: {:?}", pool_unit_balance2);

    // Remove liquidity
    let remove_liquidity_manifest = ManifestBuilder::new()
        .withdraw_from_account(
            account_component, 
            pool_unit_resource_address, 
            dec!("50")
        )
        .take_from_worktop(
            pool_unit_resource_address, 
            |builder, pool_unit_bucket| {
                builder.call_method(
                    component_address, 
                    "remove_liquidity", 
                    manifest_args!(pool_unit_bucket)
                )
            }
        )
        .assert_worktop_contains_by_amount(
            dec!("50"), 
            pool_unit_resource_address
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop)
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        remove_liquidity_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    println!("{:?}/n", receipt);

    let success = receipt.expect_commit_success();

    success.balance_changes();
}

mod utils {
    use radix_engine::types::NetworkDefinition;
    use scrypto::prelude::hash;
    use std::fs::create_dir_all;
    use std::path::{Path, PathBuf};
    use transaction::errors::TransactionValidationError;
    use transaction::manifest::{decompile, DecompileError};
    use transaction::model::TransactionManifest;
    use transaction::validation::NotarizedTransactionValidator;

    pub fn write_manifest_to_fs<P: AsRef<Path>>(
        manifest: &TransactionManifest,
        path: P,
        network_id: u8,
    ) -> Result<(), UtilsError> {
        let path = path.as_ref().to_owned();

        // Check that the path is a directory and not a file
        if path.is_file() {
            return Err(UtilsError::PathPointsToAFile(path));
        }

        // If the directory does not exist, then create it.
        create_dir_all(&path)?;

        // Decompile the transaction manifest to the manifest string and then write it to the
        // directory
        {
            let manifest_string = decompile(
                &manifest.instructions,
                &network_definition_from_network_id(network_id),
            )?;
            let manifest_path = path.join("transaction.rtm");
            std::fs::write(manifest_path, manifest_string)?;
        }

        // Write all of the blobs to the specified path
        for blob in &manifest.blobs {
            let blob_hash = hash(blob);
            let blob_path = path.join(format!("{blob_hash}.blob"));
            std::fs::write(blob_path, blob)?;
        }

        // Validate the manifest
        NotarizedTransactionValidator::validate_manifest(manifest)?;

        Ok(())
    }

    fn network_definition_from_network_id(network_id: u8) -> NetworkDefinition {
        match network_id {
            0x01 => NetworkDefinition::mainnet(),
            0x02 => NetworkDefinition {
                id: network_id,
                logical_name: "stokenet".into(),
                hrp_suffix: format!("tdx_{:x}_", network_id),
            },

            0x0A => NetworkDefinition {
                id: network_id,
                logical_name: "adapanet".into(),
                hrp_suffix: format!("tdx_{:x}_", network_id),
            },
            0x0B => NetworkDefinition {
                id: network_id,
                logical_name: "nebunet".into(),
                hrp_suffix: format!("tdx_{:x}_", network_id),
            },

            0x20 => NetworkDefinition {
                id: network_id,
                logical_name: "gilganet".into(),
                hrp_suffix: format!("tdx_{:x}_", network_id),
            },
            0x21 => NetworkDefinition {
                id: network_id,
                logical_name: "enkinet".into(),
                hrp_suffix: format!("tdx_{:x}_", network_id),
            },
            0x22 => NetworkDefinition {
                id: network_id,
                logical_name: "hammunet".into(),
                hrp_suffix: format!("tdx_{:x}_", network_id),
            },
            0x23 => NetworkDefinition {
                id: network_id,
                logical_name: "nergalnet".into(),
                hrp_suffix: format!("tdx_{:x}_", network_id),
            },
            0x24 => NetworkDefinition {
                id: network_id,
                logical_name: "mardunet".into(),
                hrp_suffix: format!("tdx_{:x}_", network_id),
            },

            0xF0 => NetworkDefinition {
                id: network_id,
                logical_name: "localnet".into(),
                hrp_suffix: "loc".into(),
            },
            0xF1 => NetworkDefinition {
                id: network_id,
                logical_name: "inttestnet".into(),
                hrp_suffix: format!("tdx_{:x}_", network_id),
            },
            0xF2 => NetworkDefinition::simulator(),

            _ => NetworkDefinition {
                id: network_id,
                logical_name: "unnamed".into(),
                hrp_suffix: format!("tdx_{:x}_", network_id),
            },
        }
    }

    #[derive(Debug)]
    pub enum UtilsError {
        PathPointsToAFile(PathBuf),
        IoError(std::io::Error),
        DecompileError(DecompileError),
        TransactionValidationError(TransactionValidationError),
    }

    impl From<std::io::Error> for UtilsError {
        fn from(value: std::io::Error) -> Self {
            Self::IoError(value)
        }
    }

    impl From<DecompileError> for UtilsError {
        fn from(value: DecompileError) -> Self {
            Self::DecompileError(value)
        }
    }

    impl From<TransactionValidationError> for UtilsError {
        fn from(value: TransactionValidationError) -> Self {
            Self::TransactionValidationError(value)
        }
    }
}