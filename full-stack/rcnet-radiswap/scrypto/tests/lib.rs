use radix_engine::ledger::ReadableSubstateStore;
use scrypto::blueprints::account::ACCOUNT_DEPOSIT_BATCH_IDENT;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use radix_engine::transaction::{TransactionReceipt, BalanceChange};
use radix_engine::types::Bech32Encoder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use utils::*;

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


/// We are setting up a testing environment with a `TestEnvironment` struct to maintain the state of the various test scenarios we will later create.
struct TestEnvironment {
    // Our TestEnvironment will have its own TestRunner to execute, manage, and view transaction, component state, and asset flows.
    test_runner: TestRunner,
    // This is the Account struct that will maintain the record of the account in which we will be making test transactions with. This Account 
    // struct contains the account's Public Key, Private Key, and account ComponentAddress.
    account: Account,
    // The PackageAddress of our Blueprint Package to instantiate our Radiswap component(s).
    package_address: PackageAddress,
    // The ResourceAddress of Token A, which will be defaulted to XRD. 
    token_a_resource_address: ResourceAddress,
    // The ResourceAddress of Token B, which will be a fungible token we will later create and mint with an initial supply of 10,000.
    token_b_resource_address: ResourceAddress,
}
/// The implementation of our TestEnvironment will set up the state of our test. It will bootstrap our environment by:
/// 1. Creating an account to sign transactions.
/// 2. Compile and locally deploy our Blueprint Package.
/// 3. Create and mint a fungible resource with a fixed supply of 10,000.
/// 4. Instantiate our first Radiswap pool component with a supply of 1,000 in each pool.
/// 
/// Setting this environment will require that these procedures run successfully before we begin creating different scenarios for our test cases.
impl TestEnvironment {
    pub fn new(mut test_runner: TestRunner) -> Self {

        // Creating a fresh new instance of our TestRunner.
        // let mut test_runner = TestRunner::builder().build();

        // Creating our first Account.
        let (public_key, private_key, account_component) = test_runner.new_allocated_account();    
        let account = Account { public_key, private_key, account_component };
        
        // Deploying our Blueprint Package Locally.
        let package_address = test_runner.compile_and_publish(this_package!());

        

        // Creating and minting Token B for testing purposes.
        let create_fungible_manifest = ManifestBuilder::new()
            .new_token_mutable(
                Default::default(), 
                AccessRule::AllowAll
            )
            .call_method(
                account_component, 
                ACCOUNT_DEPOSIT_BATCH_IDENT, 
                manifest_args!(ManifestExpression::EntireWorktop))
            .build();

        // Executing and signing the manifest to create our token for transaction submission.
        let receipt = test_runner.execute_manifest_ignoring_fee(
            create_fungible_manifest, 
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );

        // Submitting the transaction and retrieving the ResourceAddress of our Token B.
        let token_b_resource_address = receipt.expect_commit_success().new_resource_addresses()[0];
        // Defining our Token A (defaulted to XRD).
        let token_a_resource_address = RADIX_TOKEN;

        // Instantiating our Radiswap Pool Component with a default initial supply of 1,000 for each token and swap fee of 2%.
        // let manifest = ManifestBuilder::new()
        //     .call_method(
        //         account_component, 
        //         ACCOUNT_WITHDRAW_IDENT, 
        //         manifest_args!(token_a_resource_address, dec!(1000))
        //     )
        //     .call_method(
        //         account_component, 
        //         ACCOUNT_WITHDRAW_IDENT, 
        //         manifest_args!(token_b_resource_address, dec!(1000))
        //     )
        //     .take_from_worktop(token_a_resource_address, |builder, xrd_bucket| {
        //         builder.take_from_worktop(token_b_resource_address, |builder, token_b_bucket| {
        //             builder.call_function(
        //                 package_address, 
        //                 "Radiswap", 
        //                 "instantiate_radiswap", 
        //                 manifest_args!(
        //                     xrd_bucket,
        //                     token_b_bucket,
        //                     swap_fee 
        //                 )
        //             )
        //         })
        //     })
        //     .call_method(
        //         account_component, 
        //         ACCOUNT_DEPOSIT_BATCH_IDENT, 
        //         manifest_args!(ManifestExpression::EntireWorktop))
        //     .build();

        // let receipt = test_runner.execute_manifest_ignoring_fee(
        //     manifest, 
        //     vec![NonFungibleGlobalId::from_public_key(&public_key)],
        // );

        // let success = receipt.expect_commit_success();
        
        // Retrieving the ComponentAddress of our Radiswap pool component and ResourceAddress of our Pool Units resource.
        // let component_address = success.new_component_addresses()[0];
        // let pool_unit_address = success.new_resource_addresses()[1];

        Self { 
            test_runner, 
            account, 
            package_address, 
            token_a_resource_address, 
            token_b_resource_address,
        }
    }

    pub fn deploy_package(
        &mut self,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        // metadata: BTreeMap<String, MetadataEntry>,
        access_rules_config: AccessRulesConfig
    ) -> TransactionReceipt {

        let (code, schema) = Compile::compile(this_package!());

        let manifest = ManifestBuilder::new()
            .publish_package(
                code, 
                schema, 
                royalty_config, 
                // metadata,
                Default::default(), 
                access_rules_config
            )
            .build();

            util::write_manifest_to_fs(
                &manifest,
                "./deploy-package-manifest",
                0xf2,
                )
                .unwrap(); 

        let receipt = self.test_runner.execute_manifest_ignoring_fee(
            manifest, 
            vec![NonFungibleGlobalId::from_public_key(&self.account.public_key)],
        );

        return receipt
    }

    /// The methods below are methods available for our component. They generate and execute the manifest and returns a TransactionReceipt. However, each of these methods
    /// do not commit the transaction as it will be more versatile to do so in our test scenarios we will later create.

    /// This method instantiates a Radiswap pool component with inputs to determin the liquidity amount for each token and swap fee.
    /// We may use this method to instantiate multiple Radiswap pool component to test different liquidity pool configurations.
    pub fn instantiate_radiswap(
        &mut self,
        account: &Account,
        // package_address: PackageAddress, 
        token_a_address: ResourceAddress,
        token_a_amount: Decimal,
        token_b_address: ResourceAddress,
        token_b_amount: Decimal,
        swap_fee: Decimal,
    ) -> TransactionReceipt {

        // The Transaction Manifest to instantiate the Radiswap Pool Component. These instructions states the intent:
        // 1. Withdraw Token A resource (XRD) of the specified amount (token_a_amount) from the default account.
        // 2. Withdraw Token B resource of the specified amounbt (token_b_amount) from the default account.
        // 3. Take Token A resource from the worktop and place into a Bucket (xrd_bucket).
        // 4. Take Token B resource from the worktop and place into a Bucket (token_b_bucket).
        // 5. Deposit xrd_bucket and token_b_bucket into the instantiation function.
        // 6. Deposit any returned resource into our default account.
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(
                account.account_component, 
                token_a_address, 
                token_a_amount
            )
            .withdraw_from_account(
                account.account_component, 
                token_b_address, 
                token_b_amount
            )
            .take_from_worktop(token_a_address, |builder, token_a_bucket| {
                builder.take_from_worktop(token_b_address, |builder, token_b_bucket| {
                    builder.call_function(
                        self.package_address, 
                        "Radiswap", 
                        "instantiate_radiswap", 
                        manifest_args!(
                            token_a_bucket,
                            token_b_bucket,
                            swap_fee 
                        )
                    )
                })
            })
            .call_method(
                account.account_component, 
                ACCOUNT_DEPOSIT_BATCH_IDENT, 
                manifest_args!(ManifestExpression::EntireWorktop)
            )
            .build();

        // This generates the .rtm file of the Transaction Manifest.
        util::write_manifest_to_fs(
            &manifest,
            "./instantiate-radiswap-manifest",
            0xf2,
            )
            .unwrap(); 

        // Executes the manifest above and returns the Transaction Receipt.
        let receipt = self.test_runner.execute_manifest_ignoring_fee(
            manifest, 
            vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
        );

        return receipt
    }

    // pub fn inception(
    //     &mut self,
    //     account: &Account,
    //     swap_fee: Decimal,
    // ) {
    //     let amount_1: Decimal = Decimal::zero();
        
    //     let manifest = ManifestBuilder::new()
    //         .withdraw_from_account(
    //             account.account_component, 
    //             self.token_a_resource_address, 
    //             token_a_amount
    //         )
    //         .withdraw_from_account(
    //             account.account_component, 
    //             self.token_b_resource_address, 
    //             token_b_amount
    //         )
    //         .withdraw_from_account(
    //             account.account_component, 
    //             self.token_b_resource_address, 
    //             token_b_amount
    //         )
    //         .withdraw_from_account(
    //             account.account_component, 
    //             self.token_b_resource_address, 
    //             token_b_amount
    //         )
    //         .withdraw_from_account(
    //             account.account_component, 
    //             self.token_b_resource_address, 
    //             token_b_amount
    //         )
    //         .withdraw_from_account(
    //             account.account_component, 
    //             self.token_b_resource_address, 
    //             token_b_amount
    //         )
    //         .take_from_worktop_by_amount(
    //             token_a_amount,
    //             self.token_a_resource_address,
    //              |builder, xrd_bucket| {
    //             builder.take_from_worktop_by_amount(
    //                 token_b_amount,
    //                 self.token_b_resource_address, |builder, token_b_bucket| {
    //                 builder.take_from_worktop_by_amount(
    //                     amount_1,
    //                     self.token_b_resource_address, 
    //                     |builder, bucket_1| {
    //                         builder.take_from_worktop_by_amount(
    //                             amount_1, 
    //                             self.token_b_resource_address, 
    //                             |builder, bucket_2| {
    //                                 builder.take_from_worktop_by_amount(
    //                                     amount_1, 
    //                                     self.token_b_resource_address, 
    //                                     |builder, bucket_3| {
    //                                         builder.take_from_worktop_by_amount(
    //                                             amount_1, 
    //                                             self.token_b_resource_address, 
    //                                             |builder, bucket_4| {
    //                                                 builder.call_function(
    //                                                     package_address, 
    //                                                     blueprint_name, 
    //                                                     function_name, 
    //                                                     manifest_args!(
    //                                                         xrd_bucket,
    //                                                         token_b_bucket,
    //                                                         bucket_1,
    //                                                         bucket_2,
    //                                                         bucket_3,
    //                                                         bucket_4
    //                                                     )
    //                                                 )
    //                                             }
    //                                         )
    //                                     }
    //                                 )
    //                             }
    //                         )
    //                     }
    //                 )
                    
    //             })
    //         })
    //         .call_method(
    //             account.account_component, 
    //             ACCOUNT_DEPOSIT_BATCH_IDENT, 
    //             manifest_args!(ManifestExpression::EntireWorktop)
    //         )
    //         .build();

    //     // This generates the .rtm file of the Transaction Manifest.
    //     util::write_manifest_to_fs(
    //         &manifest,
    //         "./instantiate-radiswap-manifest",
    //         0xf2,
    //         )
    //         .unwrap(); 

    //     // Executes the manifest above and returns the Transaction Receipt.
    //     let receipt = self.test_runner.execute_manifest_ignoring_fee(
    //         manifest, 
    //         vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
    //     );

    //     return receipt
    // }



    /// This method specifies the Transaction Manifest which will perform a swap in our Radiswap component. We specify the ResourceAddress and amount of such
    /// resource in our method signature to provide us flexibility in what and how much token we want to swap for.
    pub fn swap(
        &mut self,
        account: &Account,
        component_address: ComponentAddress,
        input_token: ResourceAddress,
        input_amount: Decimal,
    ) -> TransactionReceipt {

        // The Transaction Manifest to perform a swap. These instructions states the intent:
        // 1. Withdraw the specified resource (input_token) and amount (input_amount) from our default account.
        // 2. Take the specified resource from the worktop and put it into a Bucket (input_bucket).
        // 3. Deposit the Bucket into our Radiswap component.
        // 4. Deposit any returned resources into our default account.
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(
                account.account_component, 
                input_token, 
                input_amount
            )
            .take_from_worktop_by_amount(input_amount, input_token, |builder, input_bucket| {
                builder.call_method(
                    component_address, 
                    "swap", 
                    manifest_args!(input_bucket)
                )
            })
            .call_method(
                account.account_component, 
                ACCOUNT_DEPOSIT_BATCH_IDENT, 
                manifest_args!(ManifestExpression::EntireWorktop)
            )
            .build();

        // This generates the .rtm file of the Transaction Manifest.
        util::write_manifest_to_fs(
            &manifest,
            "./swap-manifest",
            0xf2,
            )
            .unwrap(); 

        // Executes the manifest above and returns the Transaction Receipt.
        let receipt = self.test_runner.execute_manifest_ignoring_fee(
            manifest, 
            vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
        );

        return receipt
        
    }

    /// This method specifies the Transaction Manifest which will perform a liquidity deposit in our Radiswap component. We specify the
    /// account ComponentAddress, ResourceAddress, and amount of such resource in our method signature to provide us flexibility in the
    /// amount of Token A and Token B we would like to deposit and which account we would like to deposit from. The reason we would like 
    /// the flexibility to specify which account we would like to deposit from is to be able to determine whether the ownership of the
    /// pool will be distributed correctly.
    pub fn add_liquidity(
        &mut self,
        account: &Account,
        component_address: ComponentAddress,
        token_a_amount: Decimal,
        token_b_amount: Decimal,
    ) -> TransactionReceipt {

        // The Transaction Manifest to add liquidity to the Radiswap component. These instructions states the intent:
        // 1. Withdraw Token A and the specified amount (token_a_amount) from the specified account (account_address).
        // 2. Withdraw Token B and the specified amount (token_b_amount) from the specified account (account_address).
        // 3. Take Token A resource from the worktop and place it into a Bucket (xrd_bucket).
        // 4. Take Token B resource from the worktop and place it into a Bucket (token_b_bucket).
        // 5. Deposit xrd_bucket and token_b_bucket into the Radiswap component.
        // 6. Deposit any returned resources into our default account.
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(
                account.account_component, 
                self.token_a_resource_address, 
                token_a_amount
            )
            .withdraw_from_account(
                account.account_component, 
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
                                component_address, 
                                "add_liquidity", 
                                manifest_args!(
                                    xrd_bucket,
                                    token_b_bucket
                                ))
                        })
                })
            .call_method(
                account.account_component, 
                ACCOUNT_DEPOSIT_BATCH_IDENT, 
                manifest_args!(ManifestExpression::EntireWorktop)
            )
            .build();

        // This generates the .rtm file of the Transaction Manifest.
        util::write_manifest_to_fs(
            &manifest,
            "./add-liquidity-manifest",
            0xf2,
            )
            .unwrap(); 
    
        // Executes the manifest above and returns the Transaction Receipt.
        let receipt = self.test_runner.execute_manifest_ignoring_fee(
            manifest, 
            vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
        );

        return receipt
    }

    /// This method specifies the Transaction Manifest which will remove the accoun'ts liquidity in our Radiswap component. We specify the
    /// account ComponentAddress and amount of the Pool Units resource in our method signature to provide us flexibility of how much
    /// liquidity we would like to remove and from what account.
    pub fn remove_liquidity(
        &mut self,
        account: &Account,
        component_address: ComponentAddress,
        pool_unit_address: ResourceAddress,
        pool_units_amount: Decimal,
    ) -> TransactionReceipt {

        // The Transaction Manifest to remove liquidity from the Radiswap component. These instructions states the intent:
        // 1. Withdraw Pool Units and the specified amount (pool_units_amount) from the specified account (account).
        // 2. Take Pool Units resource from the worktop and place it into a Bucket (pool_unit_bucket).
        // 3. Deposit pool_unit_bucket into Radiswap component.
        // 4. Deposit any returned resources into our default account.
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(
                account.account_component, 
                pool_unit_address, 
                pool_units_amount
            )
            .take_from_worktop(
                pool_unit_address, 
                |builder, pool_unit_bucket| {
                    builder.call_method(
                        component_address, 
                        "remove_liquidity", 
                        manifest_args!(pool_unit_bucket)
                    )
                }
            )
            .call_method(
                account.account_component, 
                ACCOUNT_DEPOSIT_BATCH_IDENT, 
                manifest_args!(ManifestExpression::EntireWorktop)
            )
            .build();

        // This generates the .rtm file of the Transaction Manifest.
        util::write_manifest_to_fs(
            &manifest,
            "./remove-liquidity-manifest",
            0xf2,
            )
            .unwrap(); 

        // Executes the manifest above and returns the Transaction Receipt.
        let receipt = self.test_runner.execute_manifest_ignoring_fee(
            manifest, 
            vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
        );

        return receipt
    }

    pub fn test_method(&mut self, component_address: ComponentAddress) {
    

    let manifest = ManifestBuilder::new()
        .set_component_royalty_config(
            component_address, 
            RoyaltyConfigBuilder::new()
            .add_rule("buy_gumball", 1)
            .default(0)
        );
    }

    /// ********** 
    /// Below are helper methods to help us inspect the balance of our Account(s) and component as well as other utility to operate our tests.
    /// This method is used to create an Account in addition to the default account in the TestEnvironment struct. We may use this method
    /// to create another Account that we would like to sign transactions with.
    #[allow(unused)]
    pub fn new_account(&mut self) -> Account {
        let (public_key, private_key, account_component) = self.test_runner.new_allocated_account();
        
        let manifest = ManifestBuilder::new()
            // .withdraw_from_account(
            //     self.account.account_component, 
            //     self.token_b_resource_address, 
            //     dec!(1000)
            // )
            .mint_fungible(
                self.token_b_resource_address, 
                dec!(10000)
            )
            .call_method(
                account_component, 
                ACCOUNT_DEPOSIT_BATCH_IDENT, 
                manifest_args!(ManifestExpression::EntireWorktop)
            )
            .build();

        let receipt = self.test_runner.execute_manifest_ignoring_fee(
            manifest, 
            vec![NonFungibleGlobalId::from_public_key(&self.account.public_key)],
        );
        
        Account { public_key, private_key, account_component }
    }

    /// This method retrieves the balance of the resources that are relevant to our test case.
    #[allow(unused)]
    pub fn account_balance(&mut self, account_address: ComponentAddress) -> HashMap<ResourceAddress, Decimal> {
        
        let mut account_balance: HashMap<ResourceAddress, Decimal> = HashMap::new();

        let token_a_balance = self.test_runner.account_balance(account_address, self.token_a_resource_address);
        let token_b_balance = self.test_runner.account_balance(account_address, self.token_b_resource_address);
        // let pool_unit_balance = self.test_runner.account_balance(account_address, self.pool_unit_address);

        account_balance.insert(self.token_a_resource_address, token_a_balance.unwrap());
        account_balance.insert(self.token_b_resource_address, token_b_balance.unwrap());
        // account_balance.insert(self.pool_unit_address, pool_unit_balance.unwrap());

        return account_balance;
    }

    /// This method retrieves the balances of the resources the component holds.
    #[allow(unused)]
    pub fn get_vault_balance(&mut self, component_address: ComponentAddress) -> HashMap<ResourceAddress, Decimal> {
        let vault_balance = self.test_runner.get_component_resources(component_address);

        return vault_balance;
    }

    /// This method retrieves the balance of Token A the component holds.
    #[allow(unused)]
    pub fn get_vault_a_amount(&mut self, component_address: ComponentAddress) -> Decimal {
        let vault_id = self.test_runner.get_component_vaults(component_address, self.token_a_resource_address);
        let vault_a_amount = self.test_runner.inspect_vault_balance(vault_id[0]);

        return vault_a_amount.unwrap();
    }

    /// This method retrieves the balance of Token B the component holds.
    #[allow(unused)]
    pub fn get_vault_b_amount(&mut self, component_address: ComponentAddress) -> Decimal {
        let vault_id = self.test_runner.get_component_vaults(component_address, self.token_b_resource_address);
        let vault_b_amount = self.test_runner.inspect_vault_balance(vault_id[0]);

        return vault_b_amount.unwrap();
    }

    pub fn get_resource_supply(&self, resource_address: ResourceAddress) -> Decimal {

        let retrieve_substate = self.test_runner.substate_store().get_substate(
            &SubstateId(
                RENodeId::GlobalObject(
                    Address::Resource(resource_address)), 
                    NodeModuleId::SELF, 
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager)
                )
            )
            .unwrap();
    
        let resource_supply = retrieve_substate.substate.resource_manager().total_supply;

        return resource_supply
        
    }

    #[allow(unused)]
    pub fn assert_component_receive_fungible(
        &mut self, 
        receipt: &TransactionReceipt, 
        component: ComponentAddress, 
        resource_address: ResourceAddress, 
        amount: Decimal
    ) {
        let balance_changes = receipt.expect_commit_success().balance_changes();
        let account_changes = balance_changes.get(&Address::Component(component)).unwrap();
        let resource_changes = account_changes.get_key_value(&resource_address).unwrap();

        assert_eq!(
            resource_changes,
            (&resource_address, &BalanceChange::Fungible(amount)),
        );
    }

}

#[test]
fn deploy_package() {
    let test_runner = TestRunner::builder().build();
    let mut test_environment = TestEnvironment::new(test_runner);

    let royalty_config_builder = RoyaltyConfigBuilder::new()
    .add_rule("instantiate_radiswap", 10000000)
    .default(10000000);

    let mut royalty_config: BTreeMap<String, RoyaltyConfig> = BTreeMap::new();
    royalty_config.insert("Radiswap".into(), royalty_config_builder);


    let access_rules_config = AccessRulesConfig::new()
        .default(rule!(allow_all), rule!(deny_all));

    let receipt = test_environment.deploy_package(
        royalty_config, 
        access_rules_config
    );
    
    println!("{:?}/n", receipt);

    let package_address = receipt.expect_commit_success().new_package_addresses()[0];

    let meta = test_environment.test_runner.get_metadata(Address::Package(package_address), "name");

    println!("Meta: {:?}", Some(meta));

    // let new_receipt = test_environment.instantiate_radiswap(
    //     package_address, 
    //     dec!(100), 
    //     dec!(100), 
    //     dec!("0.02")
    // );

    // println!("{:?}/n", new_receipt);

    // let success = new_receipt.expect_commit_success();

    // println!("Fee: {:?}", success.fee_summary);
}

/// The purpose of our Radiswap liquidity pool is to provide the means for users to exchange between two fungible tokens.
/// Therefore, we must ensure that instantiation process can create a liquidity pool that facilitates this process.
/// The goal for testing instantiations of Radiswap pool are as follows:
/// 1. Instiatiation of the Radiswap pool component is successful.
/// 2. The two Bucket of resources that are passed must not be the same resource.
/// 3. The two Bucket of resources that are passed must be of a Fungible resource type.
/// 4. The two Bucket of resources that are passed must not be empty.
/// 5. The swap fee must be determined.
/// 6. The swap fee cannot be greater than 100% or less than 0%.
#[test]
fn instantiate_radiswap() {
    let test_runner = TestRunner::builder().build();
    let mut test_environment = TestEnvironment::new(test_runner);
    let account = test_environment.new_account();
    let token_a_address = test_environment.token_a_resource_address;
    let token_b_address = test_environment.token_b_resource_address;
    let token_a_amount = dec!(1000);
    let token_b_amount = dec!(1000);
    let swap_fee = dec!("0.02");

    let receipt = test_environment.instantiate_radiswap(
        &account,
        token_a_address,
        token_a_amount,
        token_b_address,
        token_b_amount,
        swap_fee 
    );

    receipt.expect_commit_success();    
}

/// 2. The two Bucket of resources that are passed must not be the same resource.
#[test]
fn instantiate_radiswap_fail_1() {
    let test_runner = TestRunner::builder().build();
    let mut test_environment = TestEnvironment::new(test_runner);
    let account = test_environment.new_account();
    let token_a_address = test_environment.token_a_resource_address;
    let token_b_address = test_environment.token_a_resource_address;
    let token_a_amount = dec!(1000);
    let token_b_amount = dec!(1000);
    let swap_fee = dec!("0.02");

    let receipt = test_environment.instantiate_radiswap(
        &account,
        token_a_address,
        token_a_amount,
        token_b_address,
        token_b_amount,
        swap_fee
    );

    // Can you do expect specific assertion error?
    receipt.expect_commit_failure();    
}

/// Testing Goal:
/// 1. The method actually runs - I can pass in a Bucket of resources and it returns me a Bucket of resource.
#[test]
fn swap_token_a_for_b() { 
    let test_runner = TestRunner::builder().build();
    let mut test_environment = TestEnvironment::new(test_runner);
    let account = test_environment.new_account();
    let token_a_address = test_environment.token_a_resource_address;
    let token_b_address = test_environment.token_b_resource_address;
    let token_a_amount = dec!(1000);
    let token_b_amount = dec!(1000);
    let swap_fee = dec!("0.02");

    let instantiate_radiswap = test_environment.instantiate_radiswap(
        &account,
        token_a_address,
        token_a_amount,
        token_b_address,
        token_b_amount,
        swap_fee 
    );
    
    let radiswap_address = instantiate_radiswap.expect_commit_success().new_component_addresses()[0];

    let receipt = test_environment.swap(
        &account,
        radiswap_address,
        test_environment.token_a_resource_address, 
        dec!(100)
    );

    println!("{}/n", receipt.display(&Bech32Encoder::for_simulator()));

    receipt.expect_commit_success();
}

#[test]
fn swap_token_b_for_a() {
    let test_runner = TestRunner::builder().build();
    let mut test_environment = TestEnvironment::new(test_runner);
    let account = test_environment.new_account();
    let token_a_address = test_environment.token_a_resource_address;
    let token_b_address = test_environment.token_b_resource_address;
    let token_a_amount = dec!(1000);
    let token_b_amount = dec!(1000);
    let swap_fee = dec!("0.02");

    let instantiate_radiswap = test_environment.instantiate_radiswap(
        &account,
        token_a_address,
        token_a_amount,
        token_b_address,
        token_b_amount,
        swap_fee 
    );

    let radiswap_address = instantiate_radiswap.expect_commit_success().new_component_addresses()[0];

    let input_token = test_environment.token_b_resource_address;
    let input_amount = dec!(100);

    // Creating variables for dy = (y * r * dx) / (x + r * dx) formula.
    let input_vault_amount = test_environment.get_vault_b_amount(radiswap_address);
    let output_vault_amount = test_environment.get_vault_a_amount(radiswap_address);
    
    
    // This translates to dy = (y * r * dx) / (x + r * dx)
    // This logic is used to create an assertion that the amount that is
    // returned to be is the correct amount.
    let output_amount: Decimal = (output_vault_amount
    * (dec!("1") - swap_fee)
    * input_amount)
    / (input_vault_amount + input_amount 
    * (dec!("1") - swap_fee));

    let receipt = test_environment.swap(
        &account,
        radiswap_address,
        input_token, 
        input_amount
    );

    test_environment.assert_component_receive_fungible(
        &receipt, 
        account.account_component, 
        test_environment.token_a_resource_address, 
        output_amount
    );

    println!("{}/n", receipt.display(&Bech32Encoder::for_simulator()));


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
    let test_runner = TestRunner::builder().build();
    let mut test_environment = TestEnvironment::new(test_runner);
    let account = test_environment.new_account();
    let token_a_address = test_environment.token_a_resource_address;
    let token_b_address = test_environment.token_b_resource_address;
    let token_a_amount = dec!(1000);
    let token_b_amount = dec!(1000);
    let swap_fee = dec!("0.02");

    let instantiate_radiswap = test_environment.instantiate_radiswap(
        &account,
        token_a_address,
        token_a_amount,
        token_b_address,
        token_b_amount,
        swap_fee 
    );

    let radiswap_address = instantiate_radiswap.expect_commit_success().new_component_addresses()[0];
    let pool_unit_address = instantiate_radiswap.expect_commit_success().new_resource_addresses()[1];

    let vault_a_amount = test_environment.get_vault_a_amount(radiswap_address);
    let vault_b_amount = test_environment.get_vault_b_amount(radiswap_address);

    let token_a_amount = dec!(100);
    let token_b_amount = dec!(100);

    // Logic which maintains the ratio of liquidity between two tokens when tokens are 
    // deposited to the pool.
    let (correct_amount_a, correct_amount_b) = 
        if (
            (vault_a_amount == Decimal::zero()) | (vault_b_amount == Decimal::zero())
        ) | ((vault_a_amount / vault_b_amount) == (token_a_amount / token_b_amount))
        {
            (token_a_amount, token_b_amount)
        } else if (vault_a_amount / vault_b_amount) < (token_a_amount / token_b_amount) {
            (token_b_amount * (vault_a_amount / vault_b_amount), token_b_amount)
        } else {
            (token_a_amount, token_a_amount * (token_b_amount / token_a_amount))
        };

    // Can I bring in the ResourceManager in a test environment? I can't seem to be able to retrieve total minted of a resource.
    // let pool_units_manager = borrow_resource_manager!(test_environment.pool_unit_address);
    // let pool_units_total_supply = pool_units_manager.total_supply();
    // let pool_units_amount =
    //     if pool_units_total_supply == Decimal::zero() {
    //         dec!("100.00")
    //     } else {
    //         token_a_amount * pool_units_total_supply / vault_a_amount
    //     };

    let pool_unit_supply = test_environment.get_resource_supply(pool_unit_address);

    let pool_units_amount =
        if pool_unit_supply == Decimal::zero() {
            dec!("100.00")
        } else {
            token_a_amount * pool_unit_supply / vault_a_amount
        };

    let receipt = test_environment.add_liquidity(
        &account,
        radiswap_address,
        token_a_amount, 
        token_b_amount,
    );

    println!("Transaction Receipt: {}", receipt.display(&Bech32Encoder::for_simulator()));

    test_environment.assert_component_receive_fungible(
        &receipt, 
        radiswap_address, 
        test_environment.token_a_resource_address, 
        correct_amount_a
    );

    test_environment.assert_component_receive_fungible(
        &receipt, 
        radiswap_address, 
        test_environment.token_b_resource_address, 
        correct_amount_b
    );

    test_environment.assert_component_receive_fungible(
        &receipt, 
        account.account_component, 
        pool_unit_address, 
        pool_units_amount
    );
}


#[test]
fn multiple_add_liquidity() {

    let test_runner = TestRunner::builder().build();
    let mut test_environment = TestEnvironment::new(test_runner);
    let account = test_environment.new_account();
    let token_a_address = test_environment.token_a_resource_address;
    let token_b_address = test_environment.token_b_resource_address;
    let token_a_amount = dec!(1000);
    let token_b_amount = dec!(1000);
    let swap_fee = dec!("0.02");

    let instantiate_radiswap = test_environment.instantiate_radiswap(
        &account,
        token_a_address,
        token_a_amount,
        token_b_address,
        token_b_amount,
        swap_fee 
    );

    let radiswap_address = instantiate_radiswap.expect_commit_success().new_component_addresses()[0];
    let pool_unit_address = instantiate_radiswap.expect_commit_success().new_resource_addresses()[1];    

    let account_1 = test_environment.new_account();
    let account_2 = test_environment.new_account();
    let account_3 = test_environment.new_account();

    let vector_of_accounts = vec![account_1, account_2, account_3];

    let mut vector_of_receipts = vec![];

    for account in vector_of_accounts {

        let vault_a_amount = test_environment.get_vault_a_amount(radiswap_address);
        let vault_b_amount = test_environment.get_vault_b_amount(radiswap_address);
        let token_a_amount = dec!(1000);
        let token_b_amount = dec!(1000);
    
        // Logic which maintains the ratio of liquidity between two tokens when tokens are 
        // deposited to the pool.
        let (correct_amount_a, correct_amount_b) = 
            if (
                (vault_a_amount == Decimal::zero()) | (vault_b_amount == Decimal::zero())
            ) | ((vault_a_amount / vault_b_amount) == (token_a_amount / token_b_amount))
            {
                (token_a_amount, token_b_amount)
            } else if (vault_a_amount / vault_b_amount) < (token_a_amount / token_b_amount) {
                (token_b_amount * (vault_a_amount / vault_b_amount), token_b_amount)
            } else {
                (token_a_amount, token_a_amount * (token_b_amount / token_a_amount))
            };
        
        let pool_unit_supply = test_environment.get_resource_supply(pool_unit_address);
    
        let pool_units_amount =
            if pool_unit_supply == Decimal::zero() {
                dec!("100.00")
            } else {
                token_a_amount * pool_unit_supply / vault_a_amount
            };

        let receipt = test_environment.add_liquidity(
            &account,
            radiswap_address,
            token_a_amount, 
            token_b_amount,
        );

        let display_receipt = receipt.to_string(&Bech32Encoder::for_simulator());

        vector_of_receipts.push(display_receipt);

        test_environment.assert_component_receive_fungible(
            &receipt, 
            radiswap_address, 
            test_environment.token_a_resource_address, 
            correct_amount_a
        );
    
        test_environment.assert_component_receive_fungible(
            &receipt, 
            radiswap_address, 
            test_environment.token_b_resource_address, 
            correct_amount_b
        );
    
        test_environment.assert_component_receive_fungible(
            &receipt, 
            account.account_component, 
            pool_unit_address, 
            pool_units_amount
        );
        
    }

    println!("Transaction Receipt: {}", vector_of_receipts[0]);
    println!("Transaction Receipt: {}", vector_of_receipts[1]);
    println!("Transaction Receipt: {}", vector_of_receipts[2]);

}

#[test]
fn remove_liquidity() {
    let test_runner = TestRunner::builder().build();
    let mut test_environment = TestEnvironment::new(test_runner);
    let account = test_environment.new_account();
    let token_a_address = test_environment.token_a_resource_address;
    let token_b_address = test_environment.token_b_resource_address;
    let token_a_amount = dec!(1000);
    let token_b_amount = dec!(1000);
    let swap_fee = dec!("0.02");

    let instantiate_radiswap = test_environment.instantiate_radiswap(
        &account,
        token_a_address,
        token_a_amount,
        token_b_address,
        token_b_amount,
        swap_fee 
    );

    let radiswap_address = instantiate_radiswap.expect_commit_success().new_component_addresses()[0];
    let pool_unit_address = instantiate_radiswap.expect_commit_success().new_resource_addresses()[1];

    let pool_units_amount = dec!(100);

    let pool_unit_supply = test_environment.get_resource_supply(pool_unit_address);

    let share = pool_units_amount / 
    pool_unit_supply;

    let vault_a_amount = test_environment.get_vault_a_amount(radiswap_address);
    let vault_b_amount = test_environment.get_vault_b_amount(radiswap_address);

    let token_a_returned = vault_a_amount * share;
    let token_b_returned = vault_b_amount * share;

    let receipt = test_environment.remove_liquidity(
        &account,
        radiswap_address,
        pool_unit_address,
        dec!(100), 
    );

    println!("Transaction Receipt: {}", receipt.display(&Bech32Encoder::for_simulator()));

    test_environment.assert_component_receive_fungible(
        &receipt, 
        account.account_component, 
        test_environment.token_a_resource_address, 
        token_a_returned
    );

    test_environment.assert_component_receive_fungible(
        &receipt, 
        account.account_component, 
        test_environment.token_b_resource_address, 
        token_b_returned
    );

}

#[test]
fn check_internal_state() {
    let test_runner = TestRunner::builder().build();
    let mut test_environment = TestEnvironment::new(test_runner);
    let account = test_environment.new_account();
    let token_a_address = test_environment.token_a_resource_address;
    let token_b_address = test_environment.token_b_resource_address;
    let token_a_amount = dec!(1000);
    let token_b_amount = dec!(1000);
    let swap_fee = dec!("0.02");

    let instantiate_radiswap = test_environment.instantiate_radiswap(
        &account,
        token_a_address,
        token_a_amount,
        token_b_address,
        token_b_amount,
        swap_fee 
    );

    let radiswap_address = instantiate_radiswap.expect_commit_success().new_component_addresses()[0];

    let manifest = ManifestBuilder::new()
        .call_method(
            radiswap_address, 
            "get_swap_fee", 
            manifest_args!()
        )
        .build();

    let receipt = test_environment.test_runner.execute_manifest_ignoring_fee(
        manifest, 
        vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
    );

    receipt.expect_commit_success();

    let component_swap_fee: Decimal = receipt.expect_commit_success().output(1);

    assert_eq!(
        component_swap_fee,
        swap_fee
    );

}

mod util {
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