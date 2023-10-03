use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::{builder::ManifestBuilder, prelude::TransactionManifestV1, manifest::decompiler::ManifestObjectNames};
use radix_engine::transaction::TransactionReceipt;

pub struct Account {
    public_key: Secp256k1PublicKey,
    account_address: ComponentAddress,
}

pub struct TestEnvironment {
    test_runner: DefaultTestRunner,
    account: Account,
    package_address: PackageAddress,
    resource_a: ResourceAddress,
    resource_b: ResourceAddress,
    radiswap_component: ComponentAddress,
    pool_unit: ResourceAddress,
}

impl TestEnvironment {
    // ******** Setup the environment ********
    pub fn instantiate_test() -> Self {
        let mut test_runner = TestRunnerBuilder::new().build();
    
        // Create an account
        let (public_key, _private_key, account_address) = test_runner.new_allocated_account();

        let account = Account {
            public_key,
            account_address,
        };
    
        // Publish package
        let package_address = test_runner.compile_and_publish(this_package!());
    
        // Create two fungible resources for the pool
        let resource_a = test_runner.create_fungible_resource(
            dec!(1000), 
            18u8, 
            account_address
        );
        let resource_b = test_runner.create_fungible_resource(
            dec!(1000), 
            18u8, 
            account_address
        );
        
        // Instantiate Radiswap
        let manifest = ManifestBuilder::new()
            .call_function(
                package_address, 
                "Radiswap", 
                "new", 
                manifest_args!(
                    OwnerRole::None,
                    resource_a,
                    resource_b
                )
            );
    
        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest.build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );
        
        let commit_success = receipt.expect_commit_success();

        // Define the addresses of the instantiated Radiswap component and the Radiswap pool token
        let radiswap_component = commit_success.new_component_addresses()[0];
        let pool_unit = commit_success.new_resource_addresses()[0];

        Self {
            test_runner,
            account,
            package_address,
            resource_a,
            resource_b,
            radiswap_component,
            pool_unit,
        }
    }

    // ******** Helper function to execute manifest and output the it to a file ********
    pub fn execute_manifest_ignoring_fee(
        &mut self, 
        manifest_names: ManifestObjectNames, 
        manifest: TransactionManifestV1, 
        name: &str,
        network: &NetworkDefinition
    ) -> TransactionReceipt {

        dump_manifest_to_file_system(
            manifest_names,
            &manifest,
            "./manifests",
            Some(name),
            network
        )
        .err();

        self.test_runner.execute_manifest_ignoring_fee(
            manifest, 
            vec![NonFungibleGlobalId::from_public_key(&self.account.public_key)]
        )
    }

    // ******** Test the instantiation function for Radiswap function. ********
    fn test_instantiate_radiswap(&mut self) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
        .call_function(
            self.package_address, 
            "Radiswap", 
            "new", 
            manifest_args!(
                OwnerRole::None,
                self.resource_a,
               self.resource_b
            )
        )
        .try_deposit_batch_or_abort(self.account.account_address, ManifestExpression::EntireWorktop, None);

        self.execute_manifest_ignoring_fee(
            manifest.object_names(),
            manifest.build(),
            "instantiate_radiswap",
            &NetworkDefinition::simulator()
        )
    }
    
    // ******** Test the add liquidity method. ********
    fn test_add_liquidity(&mut self) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.account.account_address,
                "withdraw",
                manifest_args!(self.resource_a, dec!(100))
            )
            .take_all_from_worktop(self.resource_a, "resource_a")
            .call_method(
                self.account.account_address,
                "withdraw",
                manifest_args!(self.resource_b, dec!(100))
            )
            .take_all_from_worktop(self.resource_b, "resource_b")
            .call_method_with_name_lookup(
                self.radiswap_component, 
                "add_liquidity", 
                |lookup | (lookup.bucket("resource_a"), lookup.bucket("resource_b")),
            )
            .try_deposit_batch_or_abort(self.account.account_address, ManifestExpression::EntireWorktop, None);

        self.execute_manifest_ignoring_fee(
            manifest.object_names(),
            manifest.build(),
            "add_liquidity",
            &NetworkDefinition::simulator()
        )
    }
        

    // ******** Test the remove liquidity method. ********
    fn test_remove_liquidity(&mut self) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.account.account_address,
                "withdraw",
                manifest_args!(self.pool_unit, dec!(10))
            )
            .take_all_from_worktop(self.pool_unit, "pool_unit")
            .call_method_with_name_lookup(
                self.radiswap_component, 
                "remove_liquidity", 
                |lookup | (lookup.bucket("pool_unit"),),
            )
            .try_deposit_batch_or_abort(self.account.account_address, ManifestExpression::EntireWorktop, None);

        self.execute_manifest_ignoring_fee(
            manifest.object_names(),
            manifest.build(),
            "remove_liquidity",
            &NetworkDefinition::simulator()
        )
    }

    // ******** Test the swap method. ********
    fn test_swap(&mut self) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.account.account_address,
                "withdraw",
                manifest_args!(self.resource_a, dec!(100))
            )
            .take_all_from_worktop(self.resource_a, "resource_in")
            .call_method_with_name_lookup(
                self.radiswap_component, 
                "swap", 
                |lookup | (lookup.bucket("resource_in"),),
            )
            .try_deposit_batch_or_abort(self.account.account_address, ManifestExpression::EntireWorktop, None);

        self.execute_manifest_ignoring_fee(
            manifest.object_names(),
            manifest.build(),
            "swap",
            &NetworkDefinition::simulator()
        )
    }
}

#[test]
fn instantiate_radiswap() {
    let mut test_environment = TestEnvironment::instantiate_test();
    let receipt = test_environment.test_instantiate_radiswap();
    receipt.expect_commit_success();
}

#[test]
fn add_liquidity() {
    let mut test_environment = TestEnvironment::instantiate_test();
    let receipt = test_environment.test_add_liquidity();
    receipt.expect_commit_success();
}

#[test]
fn remove_liquidity() {
    let mut test_environment = TestEnvironment::instantiate_test();
    test_environment.test_add_liquidity();
    let receipt = test_environment.test_remove_liquidity();
    receipt.expect_commit_success();
}

#[test]
fn swap() {
    let mut test_environment = TestEnvironment::instantiate_test();
    let receipt = test_environment.test_swap();
    receipt.expect_commit_success();
}