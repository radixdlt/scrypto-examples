use radix_engine::engine::{ModuleError, RuntimeError};
use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use radix_engine::transaction::TransactionReceipt;
use scrypto::args;
use scrypto::core::NetworkDefinition;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::TransactionManifest;
use transaction::signing::EcdsaPrivateKey;

#[test]
fn admin_can_add_shareholder() {
    // Set up environment.
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Publish package
    let package_address = test_runner.publish_package(extract_package(compile_package!()).unwrap());

    // Creating two accounts which will be used to simulate the admin and the non-admin
    let (admin_public_key, _admin_private_key, admin_component_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = test_runner.new_account();
    let (_non_admin_public_key, _non_admin_private_key, non_admin_component_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = test_runner.new_account();

    // Creating a new payment splitter component
    let instantiation_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .call_function(
                package_address,
                "PaymentSplitter",
                "instantiate_payment_splitter",
                args!(RADIX_TOKEN),
            )
            .call_method_with_all_resources(admin_component_address, "deposit_batch")
            .build();
    let instantiation_receipt: TransactionReceipt =
        test_runner.execute_manifest_ignoring_fee(instantiation_tx, vec![admin_public_key]);

    let payment_splitter_component_address: ComponentAddress = instantiation_receipt
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_component_addresses[0];
    let admin_badge_resource_address: ResourceAddress = instantiation_receipt
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_resource_addresses[0];

    // Attempting to add a new shareholder to the payment splitter
    let adding_shareholder_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .create_proof_from_account_by_amount(
                dec!("1"),
                admin_badge_resource_address,
                admin_component_address,
            )
            .call_method(
                payment_splitter_component_address,
                "add_shareholder",
                args!(dec!("20")),
            )
            .call_method_with_all_resources(non_admin_component_address, "deposit_batch")
            .build();
    let adding_shareholder_receipt: TransactionReceipt =
        test_runner.execute_manifest_ignoring_fee(adding_shareholder_tx, vec![admin_public_key]);

    adding_shareholder_receipt.expect_success();
}

#[test]
fn shareholder_cant_add_shareholder() {
    // Set up environment.
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Publish package
    let package_address = test_runner.publish_package(extract_package(compile_package!()).unwrap());

    // Creating two accounts which will be used to simulate the admin and the non-admin
    let (admin_public_key, _admin_private_key, admin_component_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = test_runner.new_account();
    let (_non_admin_public_key, _non_admin_private_key, non_admin_component_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = test_runner.new_account();

    // Creating a new payment splitter component
    let instantiation_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .call_function(
                package_address,
                "PaymentSplitter",
                "instantiate_payment_splitter",
                args!(RADIX_TOKEN),
            )
            .call_method_with_all_resources(admin_component_address, "deposit_batch")
            .build();
    let instantiation_receipt: TransactionReceipt =
        test_runner.execute_manifest_ignoring_fee(instantiation_tx, vec![admin_public_key]);

    let payment_splitter_component_address: ComponentAddress = instantiation_receipt
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_component_addresses[0];
    let admin_badge_resource_address: ResourceAddress = instantiation_receipt
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_resource_addresses[0];
    let shareholder_badge_resource_address: ResourceAddress = instantiation_receipt
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_resource_addresses[2];

    // Attempting to add a new shareholder to the payment splitter
    let adding_shareholder_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .create_proof_from_account_by_amount(
                dec!("1"),
                admin_badge_resource_address,
                admin_component_address,
            )
            .call_method(
                payment_splitter_component_address,
                "add_shareholder",
                args!(dec!("20")),
            )
            .call_method_with_all_resources(non_admin_component_address, "deposit_batch")
            .build();
    let adding_shareholder_receipt: TransactionReceipt =
        test_runner.execute_manifest_ignoring_fee(adding_shareholder_tx, vec![admin_public_key]);
    adding_shareholder_receipt.expect_success();

    // Attempting to add a new shareholder to the payment splitter
    let unauthed_adding_shareholder_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .create_proof_from_account_by_amount(
                dec!("1"),
                shareholder_badge_resource_address,
                non_admin_component_address,
            )
            .call_method(
                payment_splitter_component_address,
                "add_shareholder",
                args!(dec!("999999")),
            )
            .call_method_with_all_resources(non_admin_component_address, "deposit_batch")
            .build();
    let unauthed_adding_shareholder_receipt: TransactionReceipt = test_runner
        .execute_manifest_ignoring_fee(unauthed_adding_shareholder_tx, vec![admin_public_key]);

    // We know that we should not be authorized to add them; so, we check for an AuthorizationError
    unauthed_adding_shareholder_receipt.expect_failure(|error: &RuntimeError| -> bool {
        matches!(
            error,
            RuntimeError::ModuleError(ModuleError::AuthorizationError {
                function: _,
                authorization: _,
                error: _
            }),
        )
    });
}

#[test]
fn unauthed_cant_lock_splitter() {
    // Set up environment.
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Publish package
    let package_address = test_runner.publish_package(extract_package(compile_package!()).unwrap());

    // Creating two accounts which will be used to simulate the admin and the non-admin
    let (admin_public_key, _admin_private_key, admin_component_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = test_runner.new_account();
    let (_non_admin_public_key, _non_admin_private_key, _non_admin_component_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = test_runner.new_account();

    // Creating a new payment splitter component
    let instantiation_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .call_function(
                package_address,
                "PaymentSplitter",
                "instantiate_payment_splitter",
                args!(RADIX_TOKEN),
            )
            .call_method_with_all_resources(admin_component_address, "deposit_batch")
            .build();
    let instantiation_receipt: TransactionReceipt =
        test_runner.execute_manifest_ignoring_fee(instantiation_tx, vec![admin_public_key]);

    let payment_splitter_component_address: ComponentAddress = instantiation_receipt
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_component_addresses[0];

    // Attempting to add a new shareholder to the payment splitter
    let unauthed_locking_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .call_method(payment_splitter_component_address, "lock_splitter", args!())
            .build();
    let unauthed_locking_receipt: TransactionReceipt =
        test_runner.execute_manifest_ignoring_fee(unauthed_locking_tx, vec![admin_public_key]);

    // We know that we should not be authorized to add them; so, we check for an AuthorizationError
    unauthed_locking_receipt.expect_failure(|error: &RuntimeError| -> bool {
        matches!(
            error,
            RuntimeError::ModuleError(ModuleError::AuthorizationError {
                function: _,
                authorization: _,
                error: _
            }),
        )
    });
}

#[test]
fn admin_cant_add_shareholder_after_locking() {
    // Set up environment.
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Publish package
    let package_address = test_runner.publish_package(extract_package(compile_package!()).unwrap());

    // Creating two accounts which will be used to simulate the admin and the non-admin
    let (admin_public_key, _admin_private_key, admin_component_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = test_runner.new_account();
    let (_non_admin_public_key, _non_admin_private_key, non_admin_component_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = test_runner.new_account();

    // Creating a new payment splitter component
    let instantiation_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .call_function(
                package_address,
                "PaymentSplitter",
                "instantiate_payment_splitter",
                args!(RADIX_TOKEN),
            )
            .call_method_with_all_resources(admin_component_address, "deposit_batch")
            .build();
    let instantiation_receipt: TransactionReceipt =
        test_runner.execute_manifest_ignoring_fee(instantiation_tx, vec![admin_public_key]);

    let payment_splitter_component_address: ComponentAddress = instantiation_receipt
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_component_addresses[0];
    let admin_badge_resource_address: ResourceAddress = instantiation_receipt
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_resource_addresses[0];

    // Locking the payment splitter
    let locking_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .create_proof_from_account_by_amount(
                dec!("1"),
                admin_badge_resource_address,
                admin_component_address,
            )
            .call_method(payment_splitter_component_address, "lock_splitter", args!())
            .build();
    let _locking_receipt: TransactionReceipt =
        test_runner.execute_manifest_ignoring_fee(locking_tx, vec![admin_public_key]);

    // Attempting to add a new shareholder to the payment splitter
    let adding_shareholder_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .create_proof_from_account_by_amount(
                dec!("1"),
                admin_badge_resource_address,
                admin_component_address,
            )
            .call_method(
                payment_splitter_component_address,
                "add_shareholder",
                args!(dec!("20")),
            )
            .call_method_with_all_resources(non_admin_component_address, "deposit_batch")
            .build();
    let adding_shareholder_receipt: TransactionReceipt =
        test_runner.execute_manifest_ignoring_fee(adding_shareholder_tx, vec![admin_public_key]);

    // Adding an additional shareholder should fail.
    adding_shareholder_receipt.expect_failure(|_| true);
}

#[test]
fn anybody_can_deposit() {
    // Set up environment.
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Publish package
    let package_address = test_runner.publish_package(extract_package(compile_package!()).unwrap());

    // Creating two accounts which will be used to simulate the admin and the non-admin
    let (admin_public_key, _admin_private_key, admin_component_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = test_runner.new_account();
    let (non_admin_public_key, _non_admin_private_key, non_admin_component_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = test_runner.new_account();

    // Creating a new payment splitter component
    let instantiation_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .call_function(
                package_address,
                "PaymentSplitter",
                "instantiate_payment_splitter",
                args!(RADIX_TOKEN),
            )
            .call_method_with_all_resources(admin_component_address, "deposit_batch")
            .build();
    let instantiation_receipt: TransactionReceipt =
        test_runner.execute_manifest_ignoring_fee(instantiation_tx, vec![admin_public_key]);

    let payment_splitter_component_address: ComponentAddress = instantiation_receipt
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_component_addresses[0];

    // Depositing funds into the payment splitter
    let deposit_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .withdraw_from_account_by_amount(
                dec!("100000"),
                RADIX_TOKEN,
                non_admin_component_address,
            )
            .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                builder.call_method(
                    payment_splitter_component_address,
                    "deposit",
                    args!(Bucket(bucket_id)),
                )
            })
            .call_method_with_all_resources(non_admin_component_address, "deposit_batch")
            .build();
    let deposit_receipt: TransactionReceipt =
        test_runner.execute_manifest_ignoring_fee(deposit_tx, vec![non_admin_public_key]);

    // Adding an additional shareholder should fail.
    deposit_receipt.expect_success();
}

#[test]
fn custom_rule_splitter_works_with_correct_badges() {
    // Set up environment.
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Publish package
    let package_address = test_runner.publish_package(extract_package(compile_package!()).unwrap());

    // Creating two accounts which will be used to simulate the admin and the non-admin
    let (admin_public_key, _admin_private_key, admin_component_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = test_runner.new_account();

    // Creating multiple badges to use for the splitter testing
    let badge_creation_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .new_badge_fixed(HashMap::new(), dec!("1"))
            .new_badge_fixed(HashMap::new(), dec!("1"))
            .new_badge_fixed(HashMap::new(), dec!("1"))
            .call_method_with_all_resources(admin_component_address, "deposit_batch")
            .build();
    let badge_creation_receipt: TransactionReceipt =
        test_runner.execute_manifest_ignoring_fee(badge_creation_tx, vec![admin_public_key]);

    let supervisor_badge_resource_address: ResourceAddress = badge_creation_receipt
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_resource_addresses[0];
    let admin_badge_resource_address: ResourceAddress = badge_creation_receipt
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_resource_addresses[1];
    let superadmin_badge_resource_address: ResourceAddress = badge_creation_receipt
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_resource_addresses[2];

    // Creating the access rule which we would like to use for the addition of shareholders
    let rule: AccessRule = rule!(
        require(supervisor_badge_resource_address)
            && require(admin_badge_resource_address)
            && require(superadmin_badge_resource_address)
    );

    let instantiation_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .call_function(
                package_address,
                "PaymentSplitter",
                "instantiate_custom_access_payment_splitter",
                args!(RADIX_TOKEN, rule),
            )
            .call_method_with_all_resources(admin_component_address, "deposit_batch")
            .build();
    let instantiation_receipt: TransactionReceipt =
        test_runner.execute_manifest_ignoring_fee(instantiation_tx, vec![admin_public_key]);

    let payment_splitter_component_address: ComponentAddress = instantiation_receipt
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_component_addresses[0];

    // Attempting to add a new shareholder to the payment splitter
    let adding_shareholder_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .create_proof_from_account_by_amount(
                dec!("1"),
                admin_badge_resource_address,
                admin_component_address,
            )
            .create_proof_from_account_by_amount(
                dec!("1"),
                superadmin_badge_resource_address,
                admin_component_address,
            )
            .create_proof_from_account_by_amount(
                dec!("1"),
                supervisor_badge_resource_address,
                admin_component_address,
            )
            .call_method(
                payment_splitter_component_address,
                "add_shareholder",
                args!(dec!("20")),
            )
            .call_method_with_all_resources(admin_component_address, "deposit_batch")
            .build();
    let adding_shareholder_receipt: TransactionReceipt =
        test_runner.execute_manifest_ignoring_fee(adding_shareholder_tx, vec![admin_public_key]);
    println!("{:?}", adding_shareholder_receipt);

    // Adding an additional shareholder should fail.
    adding_shareholder_receipt.expect_success();
}

#[test]
fn custom_rule_splitter_doesnt_work_with_incorrect_badges() {
    // Set up environment.
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Publish package
    let package_address = test_runner.publish_package(extract_package(compile_package!()).unwrap());

    // Creating two accounts which will be used to simulate the admin and the non-admin
    let (admin_public_key, _admin_private_key, admin_component_address): (
        EcdsaPublicKey,
        EcdsaPrivateKey,
        ComponentAddress,
    ) = test_runner.new_account();

    // Creating multiple badges to use for the splitter testing
    let badge_creation_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .new_badge_fixed(HashMap::new(), dec!("1"))
            .new_badge_fixed(HashMap::new(), dec!("1"))
            .new_badge_fixed(HashMap::new(), dec!("1"))
            .call_method_with_all_resources(admin_component_address, "deposit_batch")
            .build();
    let badge_creation_receipt: TransactionReceipt =
        test_runner.execute_manifest_ignoring_fee(badge_creation_tx, vec![admin_public_key]);

    let supervisor_badge_resource_address: ResourceAddress = badge_creation_receipt
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_resource_addresses[0];
    let admin_badge_resource_address: ResourceAddress = badge_creation_receipt
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_resource_addresses[1];
    let superadmin_badge_resource_address: ResourceAddress = badge_creation_receipt
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_resource_addresses[2];

    // Creating the access rule which we would like to use for the addition of shareholders
    let rule: AccessRule = rule!(
        require(supervisor_badge_resource_address)
            && require(admin_badge_resource_address)
            && require(superadmin_badge_resource_address)
    );

    let instantiation_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .call_function(
                package_address,
                "PaymentSplitter",
                "instantiate_custom_access_payment_splitter",
                args!(RADIX_TOKEN, rule),
            )
            .call_method_with_all_resources(admin_component_address, "deposit_batch")
            .build();
    let instantiation_receipt: TransactionReceipt =
        test_runner.execute_manifest_ignoring_fee(instantiation_tx, vec![admin_public_key]);

    let payment_splitter_component_address: ComponentAddress = instantiation_receipt
        .result
        .get_commit_result()
        .unwrap()
        .entity_changes
        .new_component_addresses[0];

    // Attempting to add a new shareholder to the payment splitter
    let adding_shareholder_tx: TransactionManifest =
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .create_proof_from_account_by_amount(
                dec!("1"),
                admin_badge_resource_address,
                admin_component_address,
            )
            .create_proof_from_account_by_amount(
                dec!("1"),
                superadmin_badge_resource_address,
                admin_component_address,
            )
            .call_method(
                payment_splitter_component_address,
                "add_shareholder",
                args!(dec!("20")),
            )
            .call_method_with_all_resources(admin_component_address, "deposit_batch")
            .build();
    let adding_shareholder_receipt: TransactionReceipt =
        test_runner.execute_manifest_ignoring_fee(adding_shareholder_tx, vec![admin_public_key]);
    println!("{:?}", adding_shareholder_receipt);

    // Adding an additional shareholder should fail.
    adding_shareholder_receipt.expect_failure(|_| true);
}
