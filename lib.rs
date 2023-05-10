use scrypto::blueprints::account::ACCOUNT_WITHDRAW_IDENT;
use scrypto::blueprints::account::{ACCOUNT_DEPOSIT_BATCH_IDENT};
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use radix_engine::transaction::TransactionReceipt;

use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;


pub struct Account {
    pub public_key: EcdsaSecp256k1PublicKey,
    pub private_key: EcdsaSecp256k1PrivateKey,
    pub account_component: ComponentAddress,
}

/// This test instantiates a Radiswap two-token pool component from the Radiswap blueprint.
/// We are running this test to ensure:
/// 1. We can instantiate a Radiswap two-token liquidity pool with a swap fee.
/// 2. We receive an expected 100 Pool Units token to represent
/// 
/// We set this test but first creating transaction to create a fungible token with an arbitrarily determined
/// initial supply of 1,000. This token will be used as a pair with the `RADIX_TOKEN` in this liquidity pool.
/// 
/// We then create a transaction to instantiate the Radiswap component withdraw the respective tokens and deposit
/// them into the component with the swap fee.
/// 
/// Outcome:
/// 1. The transaction should succeed.
/// 2. We should receive 100 Pool Units deposited into our account.

struct TestEnvironment {
    test_runner: TestRunner,
    account: Account,
    package_address: PackageAddress,
    component_address: ComponentAddress,
    token_a_resource_address: ResourceAddress,
    token_b_resource_address: ResourceAddress,
    pool_unit_address: ResourceAddress,
}
impl TestEnvironment {
    pub fn new() -> Self {

        let mut test_runner = TestRunner::builder().build();

        let (public_key, private_key, account_component) = test_runner.new_allocated_account();    
        let account = Account { public_key, private_key, account_component };
        
        let package_address = test_runner.compile_and_publish(this_package!());

        // Creating and minting Token B for testing purposes.
        let create_fungible_manifest = ManifestBuilder::new()
            .new_token_fixed(
                Default::default(), 
                dec!("10000"))
            .call_method(
                account_component, 
                ACCOUNT_DEPOSIT_BATCH_IDENT, 
                manifest_args!(ManifestExpression::EntireWorktop))
            .build();

        let receipt = test_runner.execute_manifest_ignoring_fee(
            create_fungible_manifest, 
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );

        let token_b_resource_address = receipt.expect_commit_success().new_resource_addresses()[0];
        let token_a_resource_address = RADIX_TOKEN;

        let manifest = ManifestBuilder::new()
            .call_method(
                account_component, 
                ACCOUNT_WITHDRAW_IDENT, 
                manifest_args!(token_a_resource_address, dec!(1000))
            )
            .call_method(
                account_component, 
                ACCOUNT_WITHDRAW_IDENT, 
                manifest_args!(token_b_resource_address, dec!(1000))
            )
            .take_from_worktop(token_a_resource_address, |builder, xrd_bucket| {
                builder.take_from_worktop(token_b_resource_address, |builder, token_b_bucket| {
                    builder.call_function(
                        package_address, 
                        "Radiswap", 
                        "instantiate_radiswap", 
                        manifest_args!(
                            xrd_bucket,
                            token_b_bucket,
                            dec!("0.02") 
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
            manifest, 
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );

        let success = receipt.expect_commit_success();
        

        let component_address = success.new_component_addresses()[0];
        let pool_unit_address = success.new_resource_addresses()[1];

        Self { 
            test_runner, 
            account, 
            package_address, 
            component_address, 
            token_a_resource_address, 
            token_b_resource_address,
            pool_unit_address,
        }
    }

    pub fn instantiate_radiswap(
        &mut self, 
        token_a_amount: Decimal,
        token_b_amount: Decimal,
        swap_fee: Decimal,
    ) -> TransactionReceipt {

        let manifest = ManifestBuilder::new()
            .call_method(
                self.account.account_component, 
                ACCOUNT_WITHDRAW_IDENT, 
                manifest_args!(self.token_a_resource_address, token_a_amount)
            )
            .call_method(
                self.account.account_component, 
                ACCOUNT_WITHDRAW_IDENT, 
                manifest_args!(self.token_b_resource_address, token_b_amount)
            )
            .take_from_worktop(self.token_a_resource_address, |builder, xrd_bucket| {
                builder.take_from_worktop(self.token_b_resource_address, |builder, token_b_bucket| {
                    builder.call_function(
                        self.package_address, 
                        "Radiswap", 
                        "instantiate_radiswap", 
                        manifest_args!(
                            xrd_bucket,
                            token_b_bucket,
                            swap_fee 
                        )
                    )
                })
            })
            .call_method(
                self.account.account_component, 
                ACCOUNT_DEPOSIT_BATCH_IDENT, 
                manifest_args!(ManifestExpression::EntireWorktop))
            .build();

        utils::write_manifest_to_fs(
            &manifest,
            "./instantiate-radiswap-manifest",
            0xf2,
            )
            .unwrap(); 

        let receipt = self.test_runner.execute_manifest_ignoring_fee(
            manifest, 
            vec![NonFungibleGlobalId::from_public_key(&self.account.public_key)],
        );

        let success = receipt.expect_commit_success();
        self.component_address = success.new_component_addresses()[0];
        self.pool_unit_address = success.new_resource_addresses()[1];

        return receipt
    }
    
    #[allow(unused)]
    pub fn new_account(&mut self) -> Account {
        let (public_key, private_key, account_component) = self.test_runner.new_allocated_account();
        Account { public_key, private_key, account_component }
    }

    pub fn account_balance(&mut self, account_address: ComponentAddress) -> HashMap<ResourceAddress, Decimal> {
        
        let mut account_balance: HashMap<ResourceAddress, Decimal> = HashMap::new();

        let token_a_balance = self.test_runner.account_balance(self.account.account_component, self.token_a_resource_address);
        let token_b_balance = self.test_runner.account_balance(self.account.account_component, self.token_b_resource_address);
        let pool_unit_balance = self.test_runner.account_balance(self.account.account_component, self.pool_unit_address);

        account_balance.insert(self.token_a_resource_address, token_a_balance.unwrap());
        account_balance.insert(self.token_b_resource_address, token_b_balance.unwrap());
        account_balance.insert(self.pool_unit_address, pool_unit_balance.unwrap());

        return account_balance;
    }

    pub fn get_vault_balance(&mut self) -> HashMap<ResourceAddress, Decimal> {
        let vault_balance = self.test_runner.get_component_resources(self.component_address);

        return vault_balance;
    }

    pub fn get_vault_a_amount(&mut self) -> Decimal {
        let vault_id = self.test_runner.get_component_vaults(self.component_address, self.token_a_resource_address);
        let vault_a_amount = self.test_runner.inspect_vault_balance(vault_id[0]);

        return vault_a_amount.unwrap();
    }

    pub fn get_vault_b_amount(&mut self) -> Decimal {
        let vault_id = self.test_runner.get_component_vaults(self.component_address, self.token_b_resource_address);
        let vault_b_amount = self.test_runner.inspect_vault_balance(vault_id[0]);

        return vault_b_amount.unwrap();
    }

    pub fn swap(
        &mut self,
        input_token: ResourceAddress,
        input_amount: Decimal,
    ) -> TransactionReceipt {

        let manifest = ManifestBuilder::new()
            .call_method(
                self.account.account_component,
                ACCOUNT_WITHDRAW_IDENT,
                manifest_args!(input_token, input_amount),
            )
            .take_from_worktop_by_amount(input_amount, self.token_b_resource_address, |builder, xrd_bucket| {
                builder.call_method(
                    self.component_address, 
                    "swap", 
                    manifest_args!(xrd_bucket)
                )
            })
            .call_method(
                self.account.account_component, 
                ACCOUNT_DEPOSIT_BATCH_IDENT, 
                manifest_args!(ManifestExpression::EntireWorktop)
            )
            .build();

        utils::write_manifest_to_fs(
            &manifest,
            "./swap-manifest",
            0xf2,
            )
            .unwrap(); 

        let receipt = self.test_runner.execute_manifest_ignoring_fee(
            manifest, 
            vec![NonFungibleGlobalId::from_public_key(&self.account.public_key)],
        );

        return receipt
        
    }

    pub fn add_liquidity(
        &mut self,
        account_address: ComponentAddress,
        token_a_amount: Decimal,
        token_b_amount: Decimal,
    ) -> TransactionReceipt {

        let manifest = ManifestBuilder::new()
            .withdraw_from_account(
                account_address, 
                self.token_a_resource_address, 
                token_a_amount
            )
            .withdraw_from_account(
                account_address, 
                self.token_b_resource_address, 
                token_b_amount
            )
            .take_from_worktop(
                self.token_a_resource_address, 
                |builder, xrd_bucket| {
                    builder.take_from_worktop(
                        self.token_b_resource_address, 
                        |builder, token_b_bucket| {
                            builder.call_method(
                                self.component_address, 
                                "add_liquidity", 
                                manifest_args!(
                                    xrd_bucket,
                                    token_b_bucket
                                ))
                        })
                })
            .call_method(
                account_address, 
                ACCOUNT_DEPOSIT_BATCH_IDENT, 
                manifest_args!(ManifestExpression::EntireWorktop)
            )
            .build();

        utils::write_manifest_to_fs(
            &manifest,
            "./add-liquidity-manifest",
            0xf2,
            )
            .unwrap(); 
    

        let receipt = self.test_runner.execute_manifest_ignoring_fee(
            manifest, 
            vec![NonFungibleGlobalId::from_public_key(&self.account.public_key)],
        );

    return receipt
    }

    pub fn remove_liquidity(
        &mut self,
        account_address: ComponentAddress,
        pool_units_amount: Decimal,
    ) -> TransactionReceipt {

        let remove_liquidity_manifest = ManifestBuilder::new()
        .withdraw_from_account(
            account_address, 
            self.pool_unit_address, 
            pool_units_amount
        )
        .take_from_worktop(
            self.pool_unit_address, 
            |builder, pool_unit_bucket| {
                builder.call_method(
                    self.component_address, 
                    "remove_liquidity", 
                    manifest_args!(pool_unit_bucket)
                )
            }
        )
        .call_method(
            account_address, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop)
        )
        .build();

    let receipt = self.test_runner.execute_manifest_ignoring_fee(
        remove_liquidity_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&self.account.public_key)],
    );

    return receipt
    }


}

#[test]
fn instantiate_radiswap() {
    let mut test_environment = TestEnvironment::new();

    let receipt = test_environment.instantiate_radiswap(
        dec!(1000), 
        dec!(1000), 
        dec!("0.02")
    );

    println!("{:?}/n", receipt);
}

#[test]
fn swap_token_a_for_b() {
    let mut test_environment = TestEnvironment::new();

    let receipt = test_environment.swap(
        test_environment.token_a_resource_address, 
        dec!(100)
    );

    println!("{:?}/n", receipt);

    let vault_balance = test_environment.get_vault_balance();

    println!("Vault Balance: {:?}", vault_balance);
}

#[test]
fn swap_token_b_for_a() {
    let mut test_environment = TestEnvironment::new();

    let account_balance_before_swap = test_environment.account_balance(test_environment.account.account_component);

    let receipt = test_environment.swap(
        test_environment.token_b_resource_address, 
        dec!(100)
    );

    println!("Transaction Receipt: {:?}/n", receipt);

    let account_balance_after_swap = test_environment.account_balance(test_environment.account.account_component);

    println!("Account Balance Before Swap: {:?}/n", account_balance_before_swap);
    println!("Account Balance After Swap: {:?}/n", account_balance_after_swap);

    // Unreadable
    // println!("Transaction Result: {:?}/n", receipt.result);

    // Unreadable
    // println!("Transaction Execution Trace: {:?}/n", receipt.execution_trace);

    // Unreadable
    // println!("Resource Changes: {:?}/n", receipt.execution_trace.resource_changes);

    // Don't know what this means
    // println!("Resource Usage: {:?}/n", receipt.execution_trace.resources_usage);

    // Ideally should be in order of the manifest.
    println!("Worktop Changes: {:?}/n", receipt.execution_trace.worktop_changes()); 

    let success = receipt.expect_commit_success();

    // Unreadable
    // println!("Application Event: {:?}/n", success.application_events);

    // println!("Application Logs: {:?}/n", success.application_logs);

    // println!("Direct Vault Update: {:?}/n", success.direct_vault_updates());

    println!("Balance Changes: {:?}/n", success.balance_changes());

    // Unreadable 
    // println!("Outcome: {:?}/n", success.outcome);
    

    let pool_vault_balance = test_environment.get_vault_balance();
    println!("Vault Balance: {:?}", pool_vault_balance);
}


/// Goals for this test:
/// 1. The method can be called.
/// 2. Cannot deposit the wrong resource.
/// 3. Correct amounts are deposited.
/// 4. Amounts deposited are proportionally deposited.
/// 5. Pool Units amounts minted correctly.
/// 6. Pool Units distributed correctly.

#[test]
fn add_liquidity() {
    let mut test_environment = TestEnvironment::new();

    let default_account = test_environment.account.account_component;

    let vault_a_before = test_environment.get_vault_a_amount();
    let vault_b_before = test_environment.get_vault_b_amount();

    let token_a_amount = dec!(100);
    let token_b_amount = dec!(100);

    let receipt = test_environment.add_liquidity(
        default_account,
        token_a_amount, 
        token_b_amount,
    );

    let vault_a_after = test_environment.get_vault_a_amount();
    let vault_b_after = test_environment.get_vault_b_amount();

    println!("Transaction Receipt: {:?}", receipt);

    // At this point, goal #1 is achieved.
    let success = receipt.expect_commit_success();

    // At this point, goal #3 is achieved.
    assert_eq!(vault_a_before + token_a_amount, vault_a_after, "Token amounts deposited incorrectly");
    assert_eq!(vault_b_before + token_b_amount, vault_b_after, "Token amounts deposited incorrectly");

    // let balance_change = success.balance_changes();

    // let (mut correct_amount_a, mut correct_amount_b) = 
    //     if (
    //         (vault_a_before == Decimal::zero()) | (vault_b_before == Decimal::zero())
    //     ) | ((vault_a_before / vault_b_before) == (token_a_amount / token_b_amount))
    //     {
    //         (token_a_amount, token_b_amount)
    //     } else if (vault_a_before / vault_b_before) < (token_a_amount / token_b_amount) {
    //         (token_b_amount * (vault_a_before / vault_b_before), token_b_amount)
    //     } else {
    //         (token_a_amount, token_a_amount * (token_b_amount / token_a_amount))
    //     };

    // let deposited_a = success.balance_changes().get_index(1).unwrap().1.get(&test_environment.token_a_resource_address).unwrap().fungible().to_owned().clone();
    // let deposited_b = success.balance_changes().get_index(1).unwrap().1.get(&test_environment.token_b_resource_address).clone().unwrap().fungible().to_owned().clone();

    // assert_eq!(deposited_a, correct_amount_a, "Incorrect deposit amount");
    // assert_eq!(deposited_b, correct_amount_b, "Incorrect deposit amount");

    println!("Balance change: {:?}", success.balance_changes().get_index(1));


    // let default_account_balance = test_environment.account_balance(default_account);

}


#[test]

fn remove_liquidity() {
    let mut test_environment = TestEnvironment::new();

    let default_account = test_environment.account.account_component;

    let receipt = test_environment.remove_liquidity(
        default_account,
        dec!(100), 
    );

    println!("Transaction Receipt: {:?}", receipt);

}

// #[test]
// fn swap() {

//     let (
//         mut test_runner, 
//         public_key, 
//         account_component,
//         token_b_resource_address,
//         swap_fee,
//         component_address,
//         _pool_unit_resource_address
//     ) = self::instantiate_radiswap();

//     // Testing Swap method
//     // We will be swapping 10 Token B.
//     // To test this, we will create a formula of what amount we expect to receive and 
//     // ensure that is what we will actually receive.

//     let y_vault_id = test_runner.get_component_vaults(
//         component_address, 
//         token_b_resource_address
//     );
//     let y_vault_amount = test_runner.inspect_fungible_vault(y_vault_id[0]);
//     let x_vault_id = test_runner.get_component_vaults(
//         component_address, 
//         RADIX_TOKEN,
//     );
//     let x_vault_amount = test_runner.inspect_fungible_vault(x_vault_id[0]);
//     let dx = dec!(10);
//     let r = swap_fee;
//     let dy = 
//     (y_vault_amount.unwrap() * (dec!("1") - r) * dx) /
//      (x_vault_amount.unwrap() + (dec!("1") - r) * dx);


    // let swap_manifest = ManifestBuilder::new()
    //     .call_method(
    //         account_component,
    //         ACCOUNT_WITHDRAW_IDENT,
    //         manifest_args!(token_b_resource_address, dec!("10")),
    //     )
    //     .take_from_worktop_by_amount(dec!("10"), token_b_resource_address, |builder, xrd_bucket| {
    //         builder.call_method(
    //             component_address, 
    //             "swap", 
    //             manifest_args!(xrd_bucket)
    //         )
    //     })
    //     .assert_worktop_contains_by_amount(
    //         dy, 
    //         RADIX_TOKEN
    //     )
    //     .call_method(
    //         account_component, 
    //         ACCOUNT_DEPOSIT_BATCH_IDENT, 
    //         manifest_args!(ManifestExpression::EntireWorktop)
    //     )
    //     .build();

//     let receipt = test_runner.execute_manifest_ignoring_fee(
//         swap_manifest, 
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );

//     let xrd_balance = test_runner.account_balance(
//         account_component, 
//         RADIX_TOKEN,
//     );

//     let token_b_balance = test_runner.account_balance(
//         account_component, 
//         token_b_resource_address
//     );

//     println!("DY amount: {:?}", dy);
//     println!("XRD balance: {:?}", xrd_balance);
//     println!("Token B balance: {:?}", token_b_balance);

//     println!("{:?}/n", receipt);
//     let success = receipt.expect_commit(true);

//     success.balance_changes();
    
// }


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