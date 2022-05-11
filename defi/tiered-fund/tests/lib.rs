use radix_engine::errors::RuntimeError;
use scrypto::crypto::{EcdsaPrivateKey, EcdsaPublicKey};
use scrypto::prelude::*;

use radix_engine::ledger::*;
use radix_engine::model::{Receipt, SignedTransaction};
use radix_engine::transaction::*;
use tiered_fund::WithdrawLimit;

#[test]
pub fn withdraw_limit_arithmetic_is_correct() {
    assert_eq!(WithdrawLimit::Infinite, WithdrawLimit::Infinite);

    assert_eq!(
        WithdrawLimit::Finite(dec!("20")),
        WithdrawLimit::Finite(dec!("20"))
    );
    assert_ne!(
        WithdrawLimit::Finite(dec!("50")),
        WithdrawLimit::Finite(dec!("20"))
    );
    assert_ne!(
        WithdrawLimit::Finite(dec!("20")),
        WithdrawLimit::Finite(dec!("50"))
    );

    assert!(WithdrawLimit::Infinite > WithdrawLimit::Finite(dec!("2000000")));

    assert!(WithdrawLimit::Finite(dec!("20")) > WithdrawLimit::Finite(dec!("10")));
    assert!(WithdrawLimit::Finite(dec!("10")) < WithdrawLimit::Finite(dec!("20")));
}

#[test]
pub fn authed_methods_require_correct_badge() {
    // Setting up the ledger. Creating 2 badges which will be used for the admin auth
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut env: Environment = Environment::new(&mut ledger, 10);

    let badge1: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());
    let badge2: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());
    let access_rule: AccessRule = rule!(require(badge1) && require(badge2));

    let custom_auth_component: ComponentAddress =
        env.new_custom_tiered_fund(RADIX_TOKEN, access_rule.clone());

    // Iterating over the authed methods and ensuring that all of their transactions end with an AuthorizationError
    let authed_methods: Vec<&str> = vec![
        "add_tier",
        "remove_tier",
        "reset_tier_history",
        "update_tier_limit",
    ];
    for method_name in authed_methods.iter() {
        let method_test_tx: SignedTransaction = TransactionBuilder::new()
            .call_method(custom_auth_component, method_name, args![])
            .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.admin_account.public_key]))
            .sign([&env.admin_account.private_key]);
        let method_test_receipt: Receipt =
            env.executor.validate_and_execute(&method_test_tx).unwrap();

        let runtime_error: RuntimeError = method_test_receipt
            .result
            .expect_err("Call to authed method worked depside incorrect badges");
        assert_auth_error!(runtime_error);
    }

    // Iterating over the authed methods and ensuring that all of their transactions end with an AuthorizationError
    for method_name in authed_methods.iter() {
        let method_test_tx: SignedTransaction = TransactionBuilder::new()
            .create_proof_from_account(badge1, env.admin_account.component_address)
            .create_proof_from_account(badge2, env.admin_account.component_address)
            .call_method(custom_auth_component, method_name, args![])
            .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.admin_account.public_key]))
            .sign([&env.admin_account.private_key]);
        let method_test_receipt: Receipt =
            env.executor.validate_and_execute(&method_test_tx).unwrap();

        let runtime_error: RuntimeError = method_test_receipt
            .result
            .expect_err("Call to authed method worked depside incorrect badges");
        assert!(
            matches!(runtime_error, RuntimeError::InvokeError),
            "Authed method can't be called even by admins"
        );
    }
}

#[test]
pub fn non_authed_methods_dont_require_a_badge() {
    // Setting up the ledger. Creating 2 badges which will be used for the admin auth
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut env: Environment = Environment::new(&mut ledger, 10);

    let badge1: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());
    let badge2: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());
    let access_rule: AccessRule = rule!(require(badge1) && require(badge2));

    let custom_auth_component: ComponentAddress =
        env.new_custom_tiered_fund(RADIX_TOKEN, access_rule.clone());

    // Iterating over the authed methods and ensuring that all of their transactions end with an AuthorizationError
    let non_authed_methods: Vec<&str> = vec![
        "deposit",
        "withdraw", // This is authed but not through system level auth. This is intentionally here.
    ];
    // Iterating over the authed methods and ensuring that all of their transactions end with an AuthorizationError
    for method_name in non_authed_methods.iter() {
        let method_test_tx: SignedTransaction = TransactionBuilder::new()
            .call_method(custom_auth_component, method_name, args![])
            .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.admin_account.public_key]))
            .sign([&env.admin_account.private_key]);
        let method_test_receipt: Receipt =
            env.executor.validate_and_execute(&method_test_tx).unwrap();

        let runtime_error: RuntimeError = method_test_receipt
            .result
            .expect_err("Call to authed method worked depside incorrect badges");
        assert!(
            matches!(runtime_error, RuntimeError::InvokeError),
            "Non authed method didn't return an InvokeError"
        );
    }
}

#[test]
pub fn correct_badges_and_within_limit_succeeds() {
    // Setting up the ledger. Creating 2 badges which will be used for the admin auth
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut env: Environment = Environment::new(&mut ledger, 10);

    // ==========================================================================================
    // In this portion we setup the test parameters. To add more tests, only modify this portion
    // ==========================================================================================
    let user_badge1: ResourceAddress = env.new_token_fixed(20.into(), HashMap::new());
    let user_badge2: ResourceAddress = env.new_token_fixed(20.into(), HashMap::new());
    let user_badge3: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());

    let mut withdraw_limits: HashMap<AccessRule, WithdrawLimit> = HashMap::new();
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("10"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("100")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("12"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("500")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("15"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("1000")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("20"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Infinite,
    );

    // Defines the proofs which we will be using for the withdraw transaction
    let mut proofs_to_create: HashMap<ResourceAddress, Decimal> = HashMap::new();
    proofs_to_create.insert(user_badge1, dec!("20"));
    proofs_to_create.insert(user_badge2, dec!("10"));
    proofs_to_create.insert(user_badge3, dec!("1"));

    let withdraw_amount: Decimal = dec!("100");
    // ==========================================================================================

    let creator_badge1: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());
    let creator_badge2: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());
    let admin_access_rule: AccessRule = rule!(require(creator_badge1) && require(creator_badge2));

    let custom_auth_component: ComponentAddress =
        env.new_custom_tiered_fund(RADIX_TOKEN, admin_access_rule.clone());
    env.deposit_tokens_to_fund(
        custom_auth_component,
        env.accounts[0].clone(),
        RADIX_TOKEN,
        1_000_000.into(),
    );

    for (access_rule, withdraw_limit) in withdraw_limits.iter() {
        let add_tier_tx: SignedTransaction = TransactionBuilder::new()
            .create_proof_from_account(creator_badge1, env.admin_account.component_address)
            .create_proof_from_account(creator_badge2, env.admin_account.component_address)
            .call_method(
                custom_auth_component,
                "add_tier",
                args![access_rule.clone(), withdraw_limit.clone()],
            )
            .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.admin_account.public_key]))
            .sign([&env.admin_account.private_key]);
        let add_tier_receipt: Receipt = env.executor.validate_and_execute(&add_tier_tx).unwrap();

        assert!(add_tier_receipt.result.is_ok(), "Unable to add tier");
    }

    // Builds the transaction with the above proofs.
    let mut proof_ids: Vec<u32> = Vec::new();
    let mut transaction_builder: &mut TransactionBuilder = &mut TransactionBuilder::new();
    for (resource_address, amount) in proofs_to_create.iter() {
        transaction_builder = transaction_builder
            .create_proof_from_account_by_amount(
                amount.clone(),
                resource_address.clone(),
                env.admin_account.component_address,
            )
            .create_proof_from_auth_zone(resource_address.clone(), |builder, proof_id| {
                proof_ids.push(proof_id);
                builder
            })
    }

    let proofs: Vec<scrypto::resource::Proof> = proof_ids
        .iter()
        .map(|id| scrypto::resource::Proof(*id))
        .collect();

    let withdraw_funds_tx: SignedTransaction = transaction_builder
        .call_method(
            custom_auth_component,
            "withdraw",
            args![withdraw_amount, proofs],
        )
        .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
        .build(env.executor.get_nonce([env.admin_account.public_key]))
        .sign([&env.admin_account.private_key]);
    let withdraw_funds_receipt: Receipt = env
        .executor
        .validate_and_execute(&withdraw_funds_tx)
        .unwrap();
    assert!(withdraw_funds_receipt.result.is_ok());
}

#[test]
pub fn satisfy_two_or_more_rules_withdraws_maximum() {
    // Setting up the ledger. Creating 2 badges which will be used for the admin auth
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut env: Environment = Environment::new(&mut ledger, 10);

    // ==========================================================================================
    // In this portion we setup the test parameters. To add more tests, only modify this portion
    // ==========================================================================================
    let user_badge1: ResourceAddress = env.new_token_fixed(20.into(), HashMap::new());
    let user_badge2: ResourceAddress = env.new_token_fixed(20.into(), HashMap::new());
    let user_badge3: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());

    let mut withdraw_limits: HashMap<AccessRule, WithdrawLimit> = HashMap::new();
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("10"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("100")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("12"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("500")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("15"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("1000")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("20"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Infinite,
    );

    // Defines the proofs which we will be using for the withdraw transaction
    let mut proofs_to_create: HashMap<ResourceAddress, Decimal> = HashMap::new();
    proofs_to_create.insert(user_badge1, dec!("20"));
    proofs_to_create.insert(user_badge2, dec!("15"));
    proofs_to_create.insert(user_badge3, dec!("1"));

    let withdraw_amount: Decimal = dec!("1000");
    // ==========================================================================================

    let creator_badge1: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());
    let creator_badge2: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());
    let admin_access_rule: AccessRule = rule!(require(creator_badge1) && require(creator_badge2));

    let custom_auth_component: ComponentAddress =
        env.new_custom_tiered_fund(RADIX_TOKEN, admin_access_rule.clone());
    env.deposit_tokens_to_fund(
        custom_auth_component,
        env.accounts[0].clone(),
        RADIX_TOKEN,
        1_000_000.into(),
    );

    for (access_rule, withdraw_limit) in withdraw_limits.iter() {
        let add_tier_tx: SignedTransaction = TransactionBuilder::new()
            .create_proof_from_account(creator_badge1, env.admin_account.component_address)
            .create_proof_from_account(creator_badge2, env.admin_account.component_address)
            .call_method(
                custom_auth_component,
                "add_tier",
                args![access_rule.clone(), withdraw_limit.clone()],
            )
            .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.admin_account.public_key]))
            .sign([&env.admin_account.private_key]);
        let add_tier_receipt: Receipt = env.executor.validate_and_execute(&add_tier_tx).unwrap();

        assert!(add_tier_receipt.result.is_ok(), "Unable to add tier");
    }

    // Builds the transaction with the above proofs.
    let mut proof_ids: Vec<u32> = Vec::new();
    let mut transaction_builder: &mut TransactionBuilder = &mut TransactionBuilder::new();
    for (resource_address, amount) in proofs_to_create.iter() {
        transaction_builder = transaction_builder
            .create_proof_from_account_by_amount(
                amount.clone(),
                resource_address.clone(),
                env.admin_account.component_address,
            )
            .create_proof_from_auth_zone(resource_address.clone(), |builder, proof_id| {
                proof_ids.push(proof_id);
                builder
            })
    }

    let proofs: Vec<scrypto::resource::Proof> = proof_ids
        .iter()
        .map(|id| scrypto::resource::Proof(*id))
        .collect();

    let withdraw_funds_tx: SignedTransaction = transaction_builder
        .call_method(
            custom_auth_component,
            "withdraw",
            args![withdraw_amount, proofs],
        )
        .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
        .build(env.executor.get_nonce([env.admin_account.public_key]))
        .sign([&env.admin_account.private_key]);
    let withdraw_funds_receipt: Receipt = env
        .executor
        .validate_and_execute(&withdraw_funds_tx)
        .unwrap();

    assert!(withdraw_funds_receipt.result.is_ok());
}

#[test]
pub fn satisfy_two_or_more_rules_withdraws_maximum_and_drains_others() {
    // Setting up the ledger. Creating 2 badges which will be used for the admin auth
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut env: Environment = Environment::new(&mut ledger, 10);

    // ==========================================================================================
    // In this portion we setup the test parameters. To add more tests, only modify this portion
    // ==========================================================================================
    let user_badge1: ResourceAddress = env.new_token_fixed(20.into(), HashMap::new());
    let user_badge2: ResourceAddress = env.new_token_fixed(20.into(), HashMap::new());
    let user_badge3: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());

    let mut withdraw_limits: HashMap<AccessRule, WithdrawLimit> = HashMap::new();
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("10"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("100")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("12"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("500")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("15"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("1000")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("20"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Infinite,
    );

    // Defines the proofs which we will be using for the withdraw transaction
    let mut run1_proofs_to_create: HashMap<ResourceAddress, Decimal> = HashMap::new();
    run1_proofs_to_create.insert(user_badge1, dec!("20"));
    run1_proofs_to_create.insert(user_badge2, dec!("15"));
    run1_proofs_to_create.insert(user_badge3, dec!("1"));

    let withdraw_amount: Decimal = dec!("1000");
    // ==========================================================================================

    let creator_badge1: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());
    let creator_badge2: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());
    let admin_access_rule: AccessRule = rule!(require(creator_badge1) && require(creator_badge2));

    let custom_auth_component: ComponentAddress =
        env.new_custom_tiered_fund(RADIX_TOKEN, admin_access_rule.clone());
    env.deposit_tokens_to_fund(
        custom_auth_component,
        env.accounts[0].clone(),
        RADIX_TOKEN,
        1_000_000.into(),
    );

    for (access_rule, withdraw_limit) in withdraw_limits.iter() {
        let add_tier_tx: SignedTransaction = TransactionBuilder::new()
            .create_proof_from_account(creator_badge1, env.admin_account.component_address)
            .create_proof_from_account(creator_badge2, env.admin_account.component_address)
            .call_method(
                custom_auth_component,
                "add_tier",
                args![access_rule.clone(), withdraw_limit.clone()],
            )
            .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.admin_account.public_key]))
            .sign([&env.admin_account.private_key]);
        let add_tier_receipt: Receipt = env.executor.validate_and_execute(&add_tier_tx).unwrap();

        assert!(add_tier_receipt.result.is_ok(), "Unable to add tier");
    }

    // Builds the transaction with the above proofs.
    let mut proof_ids: Vec<u32> = Vec::new();
    let mut transaction_builder: &mut TransactionBuilder = &mut TransactionBuilder::new();
    for (resource_address, amount) in run1_proofs_to_create.iter() {
        transaction_builder = transaction_builder
            .create_proof_from_account_by_amount(
                amount.clone(),
                resource_address.clone(),
                env.admin_account.component_address,
            )
            .create_proof_from_auth_zone(resource_address.clone(), |builder, proof_id| {
                proof_ids.push(proof_id);
                builder
            })
    }

    let proofs: Vec<scrypto::resource::Proof> = proof_ids
        .iter()
        .map(|id| scrypto::resource::Proof(*id))
        .collect();

    let withdraw_funds_tx: SignedTransaction = transaction_builder
        .call_method(
            custom_auth_component,
            "withdraw",
            args![withdraw_amount, proofs],
        )
        .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
        .build(env.executor.get_nonce([env.admin_account.public_key]))
        .sign([&env.admin_account.private_key]);
    let withdraw_funds_receipt: Receipt = env
        .executor
        .validate_and_execute(&withdraw_funds_tx)
        .unwrap();
    assert!(withdraw_funds_receipt.result.is_ok());

    // Defines the proofs which we will be using for the withdraw transaction
    let mut run2_proofs_to_create: HashMap<ResourceAddress, Decimal> = HashMap::new();
    run2_proofs_to_create.insert(user_badge1, dec!("20"));
    run2_proofs_to_create.insert(user_badge2, dec!("10"));
    run2_proofs_to_create.insert(user_badge3, dec!("1"));

    let withdraw_amount: Decimal = dec!("100");

    // Builds the transaction with the above proofs.
    let mut proof_ids: Vec<u32> = Vec::new();
    let mut transaction_builder: &mut TransactionBuilder = &mut TransactionBuilder::new();
    for (resource_address, amount) in run2_proofs_to_create.iter() {
        transaction_builder = transaction_builder
            .create_proof_from_account_by_amount(
                amount.clone(),
                resource_address.clone(),
                env.admin_account.component_address,
            )
            .create_proof_from_auth_zone(resource_address.clone(), |builder, proof_id| {
                proof_ids.push(proof_id);
                builder
            })
    }

    let proofs: Vec<scrypto::resource::Proof> = proof_ids
        .iter()
        .map(|id| scrypto::resource::Proof(*id))
        .collect();

    let withdraw_funds_tx: SignedTransaction = transaction_builder
        .call_method(
            custom_auth_component,
            "withdraw",
            args![withdraw_amount, proofs],
        )
        .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
        .build(env.executor.get_nonce([env.admin_account.public_key]))
        .sign([&env.admin_account.private_key]);
    let withdraw_funds_receipt: Receipt = env
        .executor
        .validate_and_execute(&withdraw_funds_tx)
        .unwrap();

    assert!(withdraw_funds_receipt.result.is_err());
}

#[test]
pub fn badges_may_withdraw_again_after_reset() {
    // Setting up the ledger. Creating 2 badges which will be used for the admin auth
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut env: Environment = Environment::new(&mut ledger, 10);

    // ==========================================================================================
    // In this portion we setup the test parameters. To add more tests, only modify this portion
    // ==========================================================================================
    let user_badge1: ResourceAddress = env.new_token_fixed(20.into(), HashMap::new());
    let user_badge2: ResourceAddress = env.new_token_fixed(20.into(), HashMap::new());
    let user_badge3: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());

    let mut withdraw_limits: HashMap<AccessRule, WithdrawLimit> = HashMap::new();
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("10"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("100")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("12"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("500")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("15"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("1000")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("20"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Infinite,
    );

    // Defines the proofs which we will be using for the withdraw transaction
    let mut proofs_to_create: HashMap<ResourceAddress, Decimal> = HashMap::new();
    proofs_to_create.insert(user_badge1, dec!("20"));
    proofs_to_create.insert(user_badge2, dec!("10"));
    proofs_to_create.insert(user_badge3, dec!("1"));

    let withdraw_amount: Decimal = dec!("100");
    // ==========================================================================================

    let creator_badge1: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());
    let creator_badge2: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());
    let admin_access_rule: AccessRule = rule!(require(creator_badge1) && require(creator_badge2));

    let custom_auth_component: ComponentAddress =
        env.new_custom_tiered_fund(RADIX_TOKEN, admin_access_rule.clone());
    env.deposit_tokens_to_fund(
        custom_auth_component,
        env.accounts[0].clone(),
        RADIX_TOKEN,
        1_000_000.into(),
    );

    for (access_rule, withdraw_limit) in withdraw_limits.iter() {
        let add_tier_tx: SignedTransaction = TransactionBuilder::new()
            .create_proof_from_account(creator_badge1, env.admin_account.component_address)
            .create_proof_from_account(creator_badge2, env.admin_account.component_address)
            .call_method(
                custom_auth_component,
                "add_tier",
                args![access_rule.clone(), withdraw_limit.clone()],
            )
            .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.admin_account.public_key]))
            .sign([&env.admin_account.private_key]);
        let add_tier_receipt: Receipt = env.executor.validate_and_execute(&add_tier_tx).unwrap();

        assert!(add_tier_receipt.result.is_ok(), "Unable to add tier");
    }

    // Performing this twice and resetting the auth rule's history after each run to ensure that the reset of history
    // allows for more withdrawals to be made
    for _ in 0..2 {
        // Builds the transaction with the above proofs.
        let mut proof_ids: Vec<u32> = Vec::new();
        let mut transaction_builder: &mut TransactionBuilder = &mut TransactionBuilder::new();
        for (resource_address, amount) in proofs_to_create.iter() {
            transaction_builder = transaction_builder
                .create_proof_from_account_by_amount(
                    amount.clone(),
                    resource_address.clone(),
                    env.admin_account.component_address,
                )
                .create_proof_from_auth_zone(resource_address.clone(), |builder, proof_id| {
                    proof_ids.push(proof_id);
                    builder
                })
        }

        let proofs: Vec<scrypto::resource::Proof> = proof_ids
            .iter()
            .map(|id| scrypto::resource::Proof(*id))
            .collect();

        let withdraw_funds_tx: SignedTransaction = transaction_builder
            .call_method(
                custom_auth_component,
                "withdraw",
                args![withdraw_amount, proofs],
            )
            .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.admin_account.public_key]))
            .sign([&env.admin_account.private_key]);
        let withdraw_funds_receipt: Receipt = env
            .executor
            .validate_and_execute(&withdraw_funds_tx)
            .unwrap();
        assert!(withdraw_funds_receipt.result.is_ok());

        let rule_history_reset_tx: SignedTransaction = TransactionBuilder::new()
            .create_proof_from_account(creator_badge1, env.admin_account.component_address)
            .create_proof_from_account(creator_badge2, env.admin_account.component_address)
            .call_method(
                custom_auth_component,
                "reset_tier_history",
                args![
                    rule!(
                        require_amount(dec!("20"), user_badge1)
                            && require_amount(dec!("10"), user_badge2)
                            && require_amount(dec!("1"), user_badge3)
                    )
                ],
            )
            .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.admin_account.public_key]))
            .sign([&env.admin_account.private_key]);
        let rule_history_reset_receipt: Receipt = env
            .executor
            .validate_and_execute(&rule_history_reset_tx)
            .unwrap();
        assert!(rule_history_reset_receipt.result.is_ok());
    }
}

#[test]
pub fn removed_tier_can_nolonger_withdraw() {
    // Setting up the ledger. Creating 2 badges which will be used for the admin auth
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut env: Environment = Environment::new(&mut ledger, 10);

    // ==========================================================================================
    // In this portion we setup the test parameters. To add more tests, only modify this portion
    // ==========================================================================================
    let user_badge1: ResourceAddress = env.new_token_fixed(20.into(), HashMap::new());
    let user_badge2: ResourceAddress = env.new_token_fixed(20.into(), HashMap::new());
    let user_badge3: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());

    let mut withdraw_limits: HashMap<AccessRule, WithdrawLimit> = HashMap::new();
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("10"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("100")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("12"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("500")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("15"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("1000")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("20"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Infinite,
    );

    // Defines the proofs which we will be using for the withdraw transaction
    let mut proofs_to_create: HashMap<ResourceAddress, Decimal> = HashMap::new();
    proofs_to_create.insert(user_badge1, dec!("20"));
    proofs_to_create.insert(user_badge2, dec!("10"));
    proofs_to_create.insert(user_badge3, dec!("1"));

    let withdraw_amount: Decimal = dec!("100");
    // ==========================================================================================

    let creator_badge1: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());
    let creator_badge2: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());
    let admin_access_rule: AccessRule = rule!(require(creator_badge1) && require(creator_badge2));

    let custom_auth_component: ComponentAddress =
        env.new_custom_tiered_fund(RADIX_TOKEN, admin_access_rule.clone());
    env.deposit_tokens_to_fund(
        custom_auth_component,
        env.accounts[0].clone(),
        RADIX_TOKEN,
        1_000_000.into(),
    );

    for (access_rule, withdraw_limit) in withdraw_limits.iter() {
        let add_tier_tx: SignedTransaction = TransactionBuilder::new()
            .create_proof_from_account(creator_badge1, env.admin_account.component_address)
            .create_proof_from_account(creator_badge2, env.admin_account.component_address)
            .call_method(
                custom_auth_component,
                "add_tier",
                args![access_rule.clone(), withdraw_limit.clone()],
            )
            .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.admin_account.public_key]))
            .sign([&env.admin_account.private_key]);
        let add_tier_receipt: Receipt = env.executor.validate_and_execute(&add_tier_tx).unwrap();

        assert!(add_tier_receipt.result.is_ok(), "Unable to add tier");
    }

    // Removing the tier
    let remove_tier_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(creator_badge1, env.admin_account.component_address)
        .create_proof_from_account(creator_badge2, env.admin_account.component_address)
        .call_method(
            custom_auth_component,
            "remove_tier",
            args![
                rule!(
                    require_amount(dec!("20"), user_badge1)
                        && require_amount(dec!("10"), user_badge2)
                        && require_amount(dec!("1"), user_badge3)
                )
            ],
        )
        .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
        .build(env.executor.get_nonce([env.admin_account.public_key]))
        .sign([&env.admin_account.private_key]);
    let remove_tier_receipt: Receipt = env.executor.validate_and_execute(&remove_tier_tx).unwrap();

    assert!(remove_tier_receipt.result.is_ok(), "Unable to add tier");

    // Builds the transaction with the above proofs.
    let mut proof_ids: Vec<u32> = Vec::new();
    let mut transaction_builder: &mut TransactionBuilder = &mut TransactionBuilder::new();
    for (resource_address, amount) in proofs_to_create.iter() {
        transaction_builder = transaction_builder
            .create_proof_from_account_by_amount(
                amount.clone(),
                resource_address.clone(),
                env.admin_account.component_address,
            )
            .create_proof_from_auth_zone(resource_address.clone(), |builder, proof_id| {
                proof_ids.push(proof_id);
                builder
            })
    }

    let proofs: Vec<scrypto::resource::Proof> = proof_ids
        .iter()
        .map(|id| scrypto::resource::Proof(*id))
        .collect();

    let withdraw_funds_tx: SignedTransaction = transaction_builder
        .call_method(
            custom_auth_component,
            "withdraw",
            args![withdraw_amount, proofs],
        )
        .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
        .build(env.executor.get_nonce([env.admin_account.public_key]))
        .sign([&env.admin_account.private_key]);
    let withdraw_funds_receipt: Receipt = env
        .executor
        .validate_and_execute(&withdraw_funds_tx)
        .unwrap();
    println!("{:?}", withdraw_funds_receipt);
    assert!(withdraw_funds_receipt.result.is_err());
}
#[test]
pub fn more_funds_available_after_limit_increase() {
    // Setting up the ledger. Creating 2 badges which will be used for the admin auth
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    let mut env: Environment = Environment::new(&mut ledger, 10);

    // ==========================================================================================
    // In this portion we setup the test parameters. To add more tests, only modify this portion
    // ==========================================================================================
    let user_badge1: ResourceAddress = env.new_token_fixed(20.into(), HashMap::new());
    let user_badge2: ResourceAddress = env.new_token_fixed(20.into(), HashMap::new());
    let user_badge3: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());

    let mut withdraw_limits: HashMap<AccessRule, WithdrawLimit> = HashMap::new();
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("10"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("100")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("12"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("500")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("15"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Finite(dec!("1000")),
    );
    withdraw_limits.insert(
        rule!(
            require_amount(dec!("20"), user_badge1)
                && require_amount(dec!("20"), user_badge2)
                && require_amount(dec!("1"), user_badge3)
        ),
        WithdrawLimit::Infinite,
    );

    // Defines the proofs which we will be using for the withdraw transaction
    let mut proofs_to_create: HashMap<ResourceAddress, Decimal> = HashMap::new();
    proofs_to_create.insert(user_badge1, dec!("20"));
    proofs_to_create.insert(user_badge2, dec!("10"));
    proofs_to_create.insert(user_badge3, dec!("1"));

    let withdraw_amount: Decimal = dec!("100");
    // ==========================================================================================

    let creator_badge1: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());
    let creator_badge2: ResourceAddress = env.new_token_fixed(1.into(), HashMap::new());
    let admin_access_rule: AccessRule = rule!(require(creator_badge1) && require(creator_badge2));

    let custom_auth_component: ComponentAddress =
        env.new_custom_tiered_fund(RADIX_TOKEN, admin_access_rule.clone());
    env.deposit_tokens_to_fund(
        custom_auth_component,
        env.accounts[0].clone(),
        RADIX_TOKEN,
        1_000_000.into(),
    );

    for (access_rule, withdraw_limit) in withdraw_limits.iter() {
        let add_tier_tx: SignedTransaction = TransactionBuilder::new()
            .create_proof_from_account(creator_badge1, env.admin_account.component_address)
            .create_proof_from_account(creator_badge2, env.admin_account.component_address)
            .call_method(
                custom_auth_component,
                "add_tier",
                args![access_rule.clone(), withdraw_limit.clone()],
            )
            .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
            .build(env.executor.get_nonce([env.admin_account.public_key]))
            .sign([&env.admin_account.private_key]);
        let add_tier_receipt: Receipt = env.executor.validate_and_execute(&add_tier_tx).unwrap();

        assert!(add_tier_receipt.result.is_ok(), "Unable to add tier");
    }

    // Builds the transaction with the above proofs.
    let mut proof_ids: Vec<u32> = Vec::new();
    let mut transaction_builder: &mut TransactionBuilder = &mut TransactionBuilder::new();
    for (resource_address, amount) in proofs_to_create.iter() {
        transaction_builder = transaction_builder
            .create_proof_from_account_by_amount(
                amount.clone(),
                resource_address.clone(),
                env.admin_account.component_address,
            )
            .create_proof_from_auth_zone(resource_address.clone(), |builder, proof_id| {
                proof_ids.push(proof_id);
                builder
            })
    }

    let proofs: Vec<scrypto::resource::Proof> = proof_ids
        .iter()
        .map(|id| scrypto::resource::Proof(*id))
        .collect();

    let withdraw_funds_tx: SignedTransaction = transaction_builder
        .call_method(
            custom_auth_component,
            "withdraw",
            args![withdraw_amount, proofs],
        )
        .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
        .build(env.executor.get_nonce([env.admin_account.public_key]))
        .sign([&env.admin_account.private_key]);
    let withdraw_funds_receipt: Receipt = env
        .executor
        .validate_and_execute(&withdraw_funds_tx)
        .unwrap();
    println!("{:?}", withdraw_funds_receipt);
    assert!(withdraw_funds_receipt.result.is_ok());
    
    // Attempting to withdraw again to ensure failture
    let mut proof_ids: Vec<u32> = Vec::new();
    let mut transaction_builder: &mut TransactionBuilder = &mut TransactionBuilder::new();
    for (resource_address, amount) in proofs_to_create.iter() {
        transaction_builder = transaction_builder
            .create_proof_from_account_by_amount(
                amount.clone(),
                resource_address.clone(),
                env.admin_account.component_address,
            )
            .create_proof_from_auth_zone(resource_address.clone(), |builder, proof_id| {
                proof_ids.push(proof_id);
                builder
            })
    }

    let proofs: Vec<scrypto::resource::Proof> = proof_ids
        .iter()
        .map(|id| scrypto::resource::Proof(*id))
        .collect();

    let withdraw_funds_tx: SignedTransaction = transaction_builder
        .call_method(
            custom_auth_component,
            "withdraw",
            args![withdraw_amount + withdraw_amount, proofs],
        )
        .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
        .build(env.executor.get_nonce([env.admin_account.public_key]))
        .sign([&env.admin_account.private_key]);
    let withdraw_funds_receipt: Receipt = env
        .executor
        .validate_and_execute(&withdraw_funds_tx)
        .unwrap();
    println!("{:?}", withdraw_funds_receipt);
    assert!(withdraw_funds_receipt.result.is_err());

    // Increasing the limit then attempting to withdraw again
    let tier_update_tx: SignedTransaction = TransactionBuilder::new()
        .create_proof_from_account(creator_badge1, env.admin_account.component_address)
        .create_proof_from_account(creator_badge2, env.admin_account.component_address)
        .call_method(
            custom_auth_component,
            "update_tier_limit",
            args![
                rule!(
                    require_amount(dec!("20"), user_badge1)
                        && require_amount(dec!("10"), user_badge2)
                        && require_amount(dec!("1"), user_badge3)
                ),
                WithdrawLimit::Finite(dec!("200"))
            ],
        )
        .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
        .build(env.executor.get_nonce([env.admin_account.public_key]))
        .sign([&env.admin_account.private_key]);
    let tier_update_receipt: Receipt = env.executor.validate_and_execute(&tier_update_tx).unwrap();

    assert!(tier_update_receipt.result.is_ok(), "Unable to increase the tier limit");

    // Attempting to withdraw again to ensure failture
    let mut proof_ids: Vec<u32> = Vec::new();
    let mut transaction_builder: &mut TransactionBuilder = &mut TransactionBuilder::new();
    for (resource_address, amount) in proofs_to_create.iter() {
        transaction_builder = transaction_builder
            .create_proof_from_account_by_amount(
                amount.clone(),
                resource_address.clone(),
                env.admin_account.component_address,
            )
            .create_proof_from_auth_zone(resource_address.clone(), |builder, proof_id| {
                proof_ids.push(proof_id);
                builder
            })
    }

    let proofs: Vec<scrypto::resource::Proof> = proof_ids
        .iter()
        .map(|id| scrypto::resource::Proof(*id))
        .collect();

    let withdraw_funds_tx: SignedTransaction = transaction_builder
        .call_method(
            custom_auth_component,
            "withdraw",
            args![withdraw_amount, proofs],
        )
        .call_method_with_all_resources(env.admin_account.component_address, "deposit_batch")
        .build(env.executor.get_nonce([env.admin_account.public_key]))
        .sign([&env.admin_account.private_key]);
    let withdraw_funds_receipt: Receipt = env
        .executor
        .validate_and_execute(&withdraw_funds_tx)
        .unwrap();
    println!("{:?}", withdraw_funds_receipt);
    assert!(withdraw_funds_receipt.result.is_ok());
}

// =====================================================================================================================
// The environment and account structs used by this module.
// =====================================================================================================================

/// A struct which defines the environment used for testing.
pub struct Environment<'a> {
    /// The executor which will be used to run all of the transactions
    pub executor: TransactionExecutor<'a, InMemorySubstateStore>,

    /// This is the address of the package that's currently being tested.
    pub package_address: PackageAddress,

    /// This is the admin account which will be used for the bootstrapping process.
    pub admin_account: Account,

    /// These are the other accounts which were created for testing.
    pub accounts: Vec<Account>,
}

impl<'a> Environment<'a> {
    pub fn new(ledger: &'a mut InMemorySubstateStore, number_of_accounts: u8) -> Self {
        // Setting up the executor from the substate store
        let mut executor: TransactionExecutor<InMemorySubstateStore> =
            TransactionExecutor::new(ledger, false);

        // Publishing the package and getting it's address.
        let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

        // Creating the admin account
        let (pk, sk, account): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
            executor.new_account();
        let admin_account: Account = Account {
            public_key: pk,
            private_key: sk,
            component_address: account,
        };

        // Creating the required number of accounts
        let accounts: Vec<Account> = (0..number_of_accounts)
            .into_iter()
            .map(|_| {
                let (pk, sk, account): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
                    executor.new_account();
                Account {
                    public_key: pk,
                    private_key: sk,
                    component_address: account,
                }
            })
            .collect::<Vec<Account>>();

        // Creating the test environment
        Self {
            executor: executor,
            package_address: package,
            admin_account: admin_account,
            accounts: accounts,
        }
    }

    pub fn new_simple_tiered_fund(
        &mut self,
        tokens_resource_address: ResourceAddress,
    ) -> (ComponentAddress, ResourceAddress) {
        let component_instantiation_tx: SignedTransaction = TransactionBuilder::new()
            .call_function(
                self.package_address,
                "TieredFund",
                "instantiate_simple_tiered_fund",
                args![tokens_resource_address],
            )
            .call_method_with_all_resources(self.admin_account.component_address, "deposit_batch")
            .build(self.executor.get_nonce([self.admin_account.public_key]))
            .sign([&self.admin_account.private_key]);
        let component_instantiation_receipt: Receipt = self
            .executor
            .validate_and_execute(&component_instantiation_tx)
            .unwrap();
        assert!(
            component_instantiation_receipt.result.is_ok(),
            "Error when creating a simple tiered fund"
        );

        return (
            component_instantiation_receipt.new_component_addresses[0],
            component_instantiation_receipt.new_resource_addresses[0],
        );
    }

    pub fn new_custom_tiered_fund(
        &mut self,
        tokens_resource_address: ResourceAddress,
        administration_rule: AccessRule,
    ) -> ComponentAddress {
        let component_instantiation_tx: SignedTransaction = TransactionBuilder::new()
            .call_function(
                self.package_address,
                "TieredFund",
                "instantiate_custom_tiered_fund",
                args![administration_rule, tokens_resource_address],
            )
            .call_method_with_all_resources(self.admin_account.component_address, "deposit_batch")
            .build(self.executor.get_nonce([self.admin_account.public_key]))
            .sign([&self.admin_account.private_key]);
        let component_instantiation_receipt: Receipt = self
            .executor
            .validate_and_execute(&component_instantiation_tx)
            .unwrap();
        assert!(
            component_instantiation_receipt.result.is_ok(),
            "Error when creating a custom tiered fund"
        );

        return component_instantiation_receipt.new_component_addresses[0];
    }

    pub fn new_token_fixed(
        &mut self,
        supply: Decimal,
        metadata: HashMap<String, String>,
    ) -> ResourceAddress {
        let resource_auth: HashMap<ResourceMethod, (AccessRule, Mutability)> = HashMap::new();
        let new_token_tx: SignedTransaction = TransactionBuilder::new()
            .call_function(
                PackageAddress::from_str("010000000000000000000000000000000000000000000000000001")
                    .unwrap(),
                "System",
                "new_resource",
                args![
                    ResourceType::Fungible { divisibility: 18 },
                    metadata,
                    resource_auth,
                    Some(MintParams::Fungible { amount: supply })
                ],
            )
            .call_method_with_all_resources(self.admin_account.component_address, "deposit_batch")
            .build(self.executor.get_nonce([self.admin_account.public_key]))
            .sign([&self.admin_account.private_key]);
        let new_token_receipt: Receipt = self.executor.validate_and_execute(&new_token_tx).unwrap();
        assert!(
            new_token_receipt.result.is_ok(),
            "Error when creating a new fixed supply token"
        );

        return new_token_receipt.new_resource_addresses[0];
    }

    pub fn deposit_tokens_to_fund(
        &mut self,
        component_address: ComponentAddress,
        account: Account,
        resource_address: ResourceAddress,
        amount: Decimal,
    ) {
        let component_funding_tx: SignedTransaction = TransactionBuilder::new()
            .withdraw_from_account_by_amount(amount, resource_address, account.component_address)
            .take_from_worktop(resource_address, |builder, bucket_id| {
                builder.call_method(
                    component_address,
                    "deposit",
                    args![scrypto::resource::Bucket(bucket_id)],
                )
            })
            .call_method_with_all_resources(account.component_address, "deposit_batch")
            .build(self.executor.get_nonce([account.public_key]))
            .sign([&account.private_key]);
        let component_funding_receipt: Receipt = self
            .executor
            .validate_and_execute(&component_funding_tx)
            .unwrap();
        assert!(
            component_funding_receipt.result.is_ok(),
            "Funding of component has failed"
        );
    }
}

/// A struct which defines the key-pair and component addresses associate with an account component
pub struct Account {
    pub public_key: EcdsaPublicKey,
    pub private_key: EcdsaPrivateKey,
    pub component_address: ComponentAddress,
}

impl Account {
    pub fn new<T: SubstateStore>(executor: &mut TransactionExecutor<T>) -> Self {
        let (pk, sk, account): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
            executor.new_account();
        Self {
            public_key: pk,
            private_key: sk,
            component_address: account,
        }
    }
}

// Implements `Clone` for account. Can't simply derive because `EcdsaPrivateKey` does not implement clone.
impl Clone for Account {
    fn clone(&self) -> Self {
        Self {
            public_key: self.public_key.clone(),
            private_key: EcdsaPrivateKey::from_bytes(&self.private_key.to_bytes()[0..32]).unwrap(),
            component_address: self.component_address.clone(),
        }
    }
}

#[macro_export]
macro_rules! assert_auth_error {
    ($error:expr) => {{
        if !matches!(
            $error,
            RuntimeError::AuthorizationError {
                authorization: _,
                function: _,
                error: ::radix_engine::model::MethodAuthorizationError::NotAuthorized
            }
        ) {
            panic!("Expected auth error but got: {:?}", $error);
        }
    }};
}

#[test]
pub fn test_env() {
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    Environment::new(&mut ledger, 10);
}
