use scrypto::blueprints::account::ACCOUNT_WITHDRAW_IDENT;
use scrypto::blueprints::account::{AccountWithdrawInput, ACCOUNT_DEPOSIT_BATCH_IDENT};
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_instantiate() {
    let mut test_runner = TestRunner::builder().build();

    let (
        public_key, 
        _private_key, 
        account_component
    ) = test_runner.new_allocated_account();

    let package_address = test_runner.compile_and_publish(this_package!());

    let gumball_price = dec!(1);

    let manifest = ManifestBuilder::new()
        .call_function(
            package_address, 
            "GumballMachine", 
            "instantiate_gumball_machine", 
            manifest_args!(gumball_price)
        )
        .build();
 
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest.clone(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    println!("{:?}/n", receipt);    

    let success_result = receipt
    .expect_commit(true);

    let component_address = success_result.new_component_addresses()[0];

    let gumball_resource_address = success_result.new_resource_addresses()[0];

    utils::write_manifest_to_fs(&manifest, "./2-instantiate", 0xf2).unwrap();

    let manifest = ManifestBuilder::new()
        .call_method(
            account_component, 
            ACCOUNT_WITHDRAW_IDENT, 
            to_manifest_value(
                &AccountWithdrawInput {
                    amount: gumball_price,
                    resource_address: RADIX_TOKEN,
                }
            ),
        )
        .take_from_worktop_by_amount(
            gumball_price, 
            RADIX_TOKEN, 
            |builder, payment_bucket| {
                builder.call_method(
                    component_address, 
                    "buy_gumball", 
                    manifest_args!(payment_bucket)
                )
            }
        )
        .assert_worktop_contains_by_amount(gumball_price, gumball_resource_address)
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop)
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest.clone(), 
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    println!("{:?}/n", receipt);

    receipt.expect_commit_success();

    utils::write_manifest_to_fs(&manifest, "./2-buy_gumball", 0xf2).unwrap();

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