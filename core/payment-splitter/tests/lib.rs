use scrypto::api::node_modules::metadata::{MetadataEntry, MetadataValue};
use scrypto::blueprints::account::ACCOUNT_WITHDRAW_IDENT;
use scrypto::blueprints::account::{AccountWithdrawInput, ACCOUNT_DEPOSIT_BATCH_IDENT};
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;

#[test]
fn payment_splitter() {
    let mut test_runner = TestRunner::builder().build();

    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();

    let package_address = test_runner.compile_and_publish(this_package!());

    let instantiate_manifest = ManifestBuilder::new()
        .call_function(
            package_address, 
            "PaymentSplitter", 
            "instantiate_payment_splitter", 
            manifest_args!(RADIX_TOKEN)
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        instantiate_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    println!("{:?}/n", receipt);

    let success = receipt.expect_commit_success();

    let component_address = success.new_component_addresses()[0];
    let admin_badge_address = success.new_resource_addresses()[0];

    // Add Shareholder
    let add_shareholder_manifest = ManifestBuilder::new()
        .create_proof_from_account(
            account_component, 
            admin_badge_address
        )
        .call_method(
            component_address, 
            "add_shareholder", 
            manifest_args!(dec!("1")),
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop)
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        add_shareholder_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    receipt.expect_commit_success();

    // Lock Splitter

    let lock_splitter_manifest = ManifestBuilder::new()
    .create_proof_from_account(
        account_component, 
        admin_badge_address
    )
    .call_method(
        component_address, 
        "lock_splitter", 
        manifest_args!(),
    )
    .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        lock_splitter_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    receipt.expect_commit_success();

}


#[test]
fn fail_payment_splitter() {
    let mut test_runner = TestRunner::builder().build();

    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();

    let package_address = test_runner.compile_and_publish(this_package!());

    let instantiate_manifest = ManifestBuilder::new()
        .call_function(
            package_address, 
            "PaymentSplitter", 
            "instantiate_payment_splitter", 
            manifest_args!(RADIX_TOKEN)
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        instantiate_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    println!("{:?}/n", receipt);

    let success = receipt.expect_commit_success();

    let component_address = success.new_component_addresses()[0];
    let admin_badge_address = success.new_resource_addresses()[0];

    // Add Shareholder
    let add_shareholder_manifest = ManifestBuilder::new()
        .create_proof_from_account(
            account_component, 
            RADIX_TOKEN
        )
        .call_method(
            component_address, 
            "add_shareholder", 
            manifest_args!(dec!("1")),
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop)
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        add_shareholder_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    receipt.expect_commit_failure();

    // Lock Splitter

    let lock_splitter_manifest = ManifestBuilder::new()
    .create_proof_from_account(
        account_component, 
        RADIX_TOKEN
    )
    .call_method(
        component_address, 
        "lock_splitter", 
        manifest_args!(),
    )
    .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        lock_splitter_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    receipt.expect_commit_failure();

}


#[test]
fn custom_payment_splitter() {
    let mut test_runner = TestRunner::builder().build();

    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();

    let package_address = test_runner.compile_and_publish(this_package!());

    let admin_badge_manifest = ManifestBuilder::new()
        .new_badge_fixed(
            Default::default(), 
            dec!(1)
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop)
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        admin_badge_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    let admin_badge_address = receipt.expect_commit_success().new_resource_addresses()[0];

    let custom_access_rule = rule!(require(admin_badge_address));

    let instantiate_manifest = ManifestBuilder::new()
        .call_function(
            package_address, 
            "PaymentSplitter", 
            "instantiate_custom_access_payment_splitter", 
            manifest_args!(
                RADIX_TOKEN,
                custom_access_rule
            )
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        instantiate_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    println!("{:?}/n", receipt);

    let success = receipt.expect_commit_success();

    let component_address = success.new_component_addresses()[0];

    // Add Shareholder
    let add_shareholder_manifest = ManifestBuilder::new()
        .create_proof_from_account(
            account_component, 
            admin_badge_address
        )
        .call_method(
            component_address, 
            "add_shareholder", 
            manifest_args!(dec!("1")),
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop)
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        add_shareholder_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    receipt.expect_commit_success();

    // Lock Splitter

    let lock_splitter_manifest = ManifestBuilder::new()
    .create_proof_from_account(
        account_component, 
        admin_badge_address
    )
    .call_method(
        component_address, 
        "lock_splitter", 
        manifest_args!(),
    )
    .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        lock_splitter_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    receipt.expect_commit_success();
}

#[test]
fn custom_require_any_of() {
    let mut test_runner = TestRunner::builder().build();

    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();

    let package_address = test_runner.compile_and_publish(this_package!());

    let admin_badge_manifest = ManifestBuilder::new()
        .new_badge_fixed(
            Default::default(), 
            dec!(1)
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop)
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        admin_badge_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    let admin_badge_address = receipt.expect_commit_success().new_resource_addresses()[0];

    let admin_badge_manifest2 = ManifestBuilder::new()
    .new_badge_fixed(
        Default::default(), 
        dec!(1)
    )
    .call_method(
        account_component, 
        ACCOUNT_DEPOSIT_BATCH_IDENT, 
        manifest_args!(ManifestExpression::EntireWorktop)
    )
    .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        admin_badge_manifest2, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    let admin_badge_address2 = receipt.expect_commit_success().new_resource_addresses()[0];

    let custom_access_rule = rule!(
        require_any_of(vec![admin_badge_address,
                admin_badge_address2
            ]
        )
    );

    let instantiate_manifest = ManifestBuilder::new()
        .call_function(
            package_address, 
            "PaymentSplitter", 
            "instantiate_custom_access_payment_splitter", 
            manifest_args!(
                RADIX_TOKEN,
                custom_access_rule
            )
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        instantiate_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    println!("{:?}/n", receipt);

    let success = receipt.expect_commit_success();

    let component_address = success.new_component_addresses()[0];

    // Add Shareholder
    let add_shareholder_manifest = ManifestBuilder::new()
        .create_proof_from_account(
            account_component, 
            admin_badge_address
        )
        .call_method(
            component_address, 
            "add_shareholder", 
            manifest_args!(dec!("1")),
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop)
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        add_shareholder_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    receipt.expect_commit_success();

    // Lock Splitter

    let lock_splitter_manifest = ManifestBuilder::new()
    .create_proof_from_account(
        account_component, 
        admin_badge_address2
    )
    .call_method(
        component_address, 
        "lock_splitter", 
        manifest_args!(),
    )
    .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        lock_splitter_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    receipt.expect_commit_success();
}

#[test]
fn custom_require_amount() {
    let mut test_runner = TestRunner::builder().build();

    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();

    let package_address = test_runner.compile_and_publish(this_package!());

    let admin_badge_manifest = ManifestBuilder::new()
        .new_badge_fixed(
            Default::default(), 
            dec!(1)
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop)
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        admin_badge_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    let admin_badge_address = receipt.expect_commit_success().new_resource_addresses()[0];

    let custom_access_rule = rule!(require_amount(dec!("2"), admin_badge_address));

    let instantiate_manifest = ManifestBuilder::new()
        .call_function(
            package_address, 
            "PaymentSplitter", 
            "instantiate_custom_access_payment_splitter", 
            manifest_args!(
                RADIX_TOKEN,
                custom_access_rule
            )
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        instantiate_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    println!("{:?}/n", receipt);

    let success = receipt.expect_commit_success();

    let component_address = success.new_component_addresses()[0];

    // Add Shareholder
    let add_shareholder_manifest = ManifestBuilder::new()
        .create_proof_from_account(
            account_component, 
            admin_badge_address
        )
        .pop_from_auth_zone(
            |builder, admin_badge_proof| {
            builder.clone_proof(
                &admin_badge_proof,
                 |builder, admin_badge_proof2| {
                    builder.push_to_auth_zone(admin_badge_proof2);
                    builder.push_to_auth_zone(admin_badge_proof)
                 })
        })
        .call_method(
            component_address, 
            "add_shareholder", 
            manifest_args!(dec!("1")),
        )
        .call_method(
            account_component, 
            ACCOUNT_DEPOSIT_BATCH_IDENT, 
            manifest_args!(ManifestExpression::EntireWorktop)
        )
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        add_shareholder_manifest, 
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    receipt.expect_commit_success();

    // Lock Splitter

    // let lock_splitter_manifest = ManifestBuilder::new()
    // .create_proof_from_account(
    //     account_component, 
    //     admin_badge_address
    // )
    // .call_method(
    //     component_address, 
    //     "lock_splitter", 
    //     manifest_args!(),
    // )
    // .build();

    // let receipt = test_runner.execute_manifest_ignoring_fee(
    //     lock_splitter_manifest, 
    //     vec![NonFungibleGlobalId::from_public_key(&public_key)]
    // );

    // receipt.expect_commit_success();
}

