use radix_engine::transaction::TransactionReceipt;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::{
    builder::ManifestBuilder, manifest::decompiler::ManifestObjectNames,
    prelude::TransactionManifestV1,
};

pub struct Account {
    public_key: Secp256k1PublicKey,
    account_address: ComponentAddress,
}

pub struct TestEnvironment {
    test_runner: DefaultTestRunner,
    account: Account,
    package_address: PackageAddress,
}

impl TestEnvironment {
    pub fn instantiate_test() -> Self {
        let mut test_runner = TestRunnerBuilder::new().build();

        // Create an account
        let (public_key, _private_key, account_address) = test_runner.new_allocated_account();

        let account = Account {
            public_key,
            account_address,
        };

        let package_address = test_runner.compile_and_publish(this_package!());

        Self {
            test_runner,
            account,
            package_address,
        }
    }

    pub fn execute_manifest_ignoring_fee(
        &mut self,
        manifest_names: ManifestObjectNames,
        manifest: TransactionManifestV1,
        name: &str,
        network: &NetworkDefinition,
    ) -> TransactionReceipt {
        dump_manifest_to_file_system(
            manifest_names,
            &manifest,
            "./transaction_manifest/fixed_price_sale",
            Some(name),
            network,
        )
        .err();

        self.test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &self.account.public_key,
            )],
        )
    }

    pub fn instantiate_fixed_price_sale(
        &mut self,
        non_fungible_tokens: ResourceAddress,
        accepted_payment_token: ResourceAddress,
        price: Decimal,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .withdraw_non_fungibles_from_account(
                self.account.account_address,
                non_fungible_tokens,
                btreeset!(NonFungibleLocalId::integer(1)),
            )
            .take_all_from_worktop(non_fungible_tokens, "bucket")
            .call_function_with_name_lookup(
                self.package_address,
                "FixedPriceSale",
                "instantiate_fixed_price_sale",
                |lookup| (vec![lookup.bucket("bucket")], accepted_payment_token, price),
            )
            .deposit_batch(self.account.account_address);

        self.execute_manifest_ignoring_fee(
            manifest.object_names(),
            manifest.build(),
            "instantiate_fixed_price_sale",
            &NetworkDefinition::simulator(),
        )
    }
}

#[test]
fn instantiate_fixed_price_sale() {
    let mut test_environment = TestEnvironment::instantiate_test();

    let non_fungible_token = test_environment
        .test_runner
        .create_non_fungible_resource(test_environment.account.account_address);

    let receipt = test_environment.instantiate_fixed_price_sale(non_fungible_token, XRD, dec!(10));

    receipt.expect_commit_success();
}

// To be continued
