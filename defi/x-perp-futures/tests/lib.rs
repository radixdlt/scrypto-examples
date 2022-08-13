use scrypto::crypto::{EcdsaPublicKey, EcdsaPrivateKey};
use radix_engine::transaction::*;
use radix_engine::ledger::*;
use scrypto::prelude::*;

use x_perp_futures::Position;
use x_perp_futures::PositionType;

struct TestEnv<'a, L: SubstateStore> {
    executor: TransactionExecutor<'a, L>,
    pk: EcdsaPublicKey,
    sk: EcdsaPrivateKey,
    account: ComponentAddress,
    usd: ResourceAddress,
    clearing_house: ComponentAddress,
}

fn set_up_test_env<'a, L: SubstateStore>(ledger: &'a mut L) -> TestEnv<'a, L> {
    let mut executor = TransactionExecutor::new(ledger, false);
    let (pk, sk, account) = executor.new_account();
    let package = executor.publish_package(compile_package!()).unwrap();

    let receipt = executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .new_token_fixed(HashMap::new(), dec!("1000000"))
                .call_method_with_all_resources(account, "deposit_batch")
                .build(executor.get_nonce([pk]))
                .sign([&sk])
        )
        .unwrap();
    let usd = receipt.new_resource_addresses[0];

    let receipt = executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .call_function(
                    package,
                    "ClearingHouse",
                    "instantiate_clearing_house",
                    args![
                        usd,
                        dec!("1"),
                        dec!("99999")
                    ]
                )
                .call_method_with_all_resources(account, "deposit_batch")
                .build(executor.get_nonce([pk]))
                .sign([&sk])
        )
        .unwrap();
    let clearing_house = receipt.new_component_addresses[0];

    TestEnv {
        executor,
        pk,
        sk,
        account,
        usd,
        clearing_house,
    }
}

fn create_user<'a, L: SubstateStore>(env: &mut TestEnv<'a, L>) -> ResourceAddress {
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .call_method(env.clearing_house, "new_user", to_struct!())
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.pk]))
                .sign([&env.sk])
        )
        .unwrap();
    assert!(receipt.result.is_ok());
    receipt.new_resource_addresses[0]
}

fn get_position<'a, L: SubstateStore>(env: &mut TestEnv<'a, L>, user_id: ResourceAddress, nth: usize) -> Position {
    let mut receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .call_method(
                    env.clearing_house,
                    "get_position",
                    args![
                        user_id,
                        nth
                    ]
                )
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.pk]))
                .sign([&env.sk])
        )
        .unwrap();
    assert!(receipt.result.is_ok());
    let encoded = receipt.outputs.swap_remove(0).raw;
    scrypto_decode(&encoded).unwrap()
}

#[test]
fn test_long() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = set_up_test_env(&mut ledger);

    let user1 = create_user(&mut env);
    let user2 = create_user(&mut env);

    // First, user1 longs BTC with 500 USD x4
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .create_proof_from_account_by_amount(dec!("1"), user1, env.account)
                .withdraw_from_account_by_amount(dec!("500"), env.usd, env.account)
                .create_proof_from_auth_zone(user1, |builder, proof_id| {
                    builder.take_from_worktop(env.usd, |builder, bucket_id| {
                        builder.call_method(
                            env.clearing_house,
                            "new_position",
                            args![
                                scrypto::resource::Proof(proof_id),
                                scrypto::resource::Bucket(bucket_id),
                                dec!("4"),
                                String::from("Long")
                            ],
                        )
                    })
                })
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.pk]))
                .sign([&env.sk])
        )
        .unwrap();
    println!("{:?}", receipt);
    let position = get_position(&mut env, user1, 0);
    assert_eq!(
        position,
        Position {
            position_type: PositionType::Long,
            margin_in_quote: "500".parse().unwrap(),
            leverage: "4".parse().unwrap(),
            position_in_base: "0.019608035372895813".parse().unwrap()
        }
    );

    // First, user2 longs BTC with 500 USD x1
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .create_proof_from_account_by_amount(dec!("1"), user2, env.account)
                .withdraw_from_account_by_amount(dec!("500"), env.usd, env.account)
                .create_proof_from_auth_zone(user2, |builder, proof_id| {
                    builder.take_from_worktop(env.usd, |builder, bucket_id| {
                        builder.call_method(
                            env.clearing_house,
                            "new_position",
                            args![
                                scrypto::resource::Proof(proof_id),
                                scrypto::resource::Bucket(bucket_id),
                                dec!("4"),
                                String::from("Long")
                            ],
                        )
                    })
                })
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.pk]))
                .sign([&env.sk])
        )
        .unwrap();
    println!("{:?}", receipt);
    let position = get_position(&mut env, user2, 0);
    assert_eq!(
        position,
        Position {
            position_type: PositionType::Long,
            margin_in_quote: "500".parse().unwrap(),
            leverage: "4".parse().unwrap(),
            position_in_base: "0.018853872914683876".parse().unwrap()
        }
    );

    // user1 settles his position
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .create_proof_from_account(user1, env.account)
                .create_proof_from_auth_zone(user1, |builder, proof_id| {
                    builder.call_method(
                        env.clearing_house,
                        "settle_position",
                        args![
                            scrypto::resource::Proof(proof_id),
                            0usize
                        ]
                    )
                })
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.pk]))
                .sign([&env.sk])
        )
        .unwrap();
    println!("{:?}", receipt);
}

#[test]
fn test_short() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = set_up_test_env(&mut ledger);

    let user1 = create_user(&mut env);
    let user2 = create_user(&mut env);

    // First, user1 shorts BTC with 500 USD x4
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .create_proof_from_account_by_amount(dec!("1"), user1, env.account)
                .withdraw_from_account_by_amount(dec!("500"), env.usd, env.account)
                .create_proof_from_auth_zone(user1, |builder, proof_id| {
                    builder.take_from_worktop(env.usd, |builder, bucket_id| {
                        builder.call_method(
                            env.clearing_house,
                            "new_position",
                            args![
                                scrypto::resource::Proof(proof_id),
                                scrypto::resource::Bucket(bucket_id),
                                dec!("4"),
                                String::from("Short")
                            ],
                        )
                    })
                })
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.pk]))
                .sign([&env.sk])
        )
        .unwrap();
    println!("{:?}", receipt);
    let position = get_position(&mut env, user1, 0);
    assert_eq!(
        position,
        Position {
            position_type: PositionType::Short,
            margin_in_quote: "500".parse().unwrap(),
            leverage: "4".parse().unwrap(),
            position_in_base: "-0.02040837151399504".parse().unwrap()
        }
    );

    // First, user2 shorts BTC with 500 USD x1
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .create_proof_from_account_by_amount(dec!("1"), user2, env.account)
                .withdraw_from_account_by_amount(dec!("500"), env.usd, env.account)
                .create_proof_from_auth_zone(user2, |builder, proof_id| {
                    builder.take_from_worktop(env.usd, |builder, bucket_id| {
                        builder.call_method(
                            env.clearing_house,
                            "new_position",
                            args![
                                scrypto::resource::Proof(proof_id),
                                scrypto::resource::Bucket(bucket_id),
                                dec!("4"),
                                String::from("Short")
                            ],
                        )
                    })
                })
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.pk]))
                .sign([&env.sk])
        )
        .unwrap();
    println!("{:?}", receipt);
    let position = get_position(&mut env, user2, 0);
    assert_eq!(
        position,
        Position {
            position_type: PositionType::Short,
            margin_in_quote: "500".parse().unwrap(),
            leverage: "4".parse().unwrap(),
            position_in_base: "-0.021258729184970573".parse().unwrap()
        }
    );

    // user1 settles his position
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .create_proof_from_account(user1, env.account)
                .create_proof_from_auth_zone(user1, |builder, proof_id| {
                    builder.call_method(
                        env.clearing_house,
                        "settle_position",
                        args![
                            scrypto::resource::Proof(proof_id),
                            0usize
                        ]
                    )
                })
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.pk]))
                .sign([&env.sk])
        )
        .unwrap();
    println!("{:?}", receipt);
}
