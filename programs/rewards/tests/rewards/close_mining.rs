use crate::utils::{assert_custom_on_chain_error::AssertCustomOnChainErr, *};
use mplx_rewards::{error::MplxRewardsError, utils::LockupPeriod};
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{clock::SECONDS_PER_DAY, signature::Keypair, signer::Signer};

async fn setup() -> (ProgramTestContext, TestRewards, Keypair, Pubkey) {
    let test = ProgramTest::new(
        "mplx_rewards",
        mplx_rewards::id(),
        processor!(mplx_rewards::processor::process_instruction),
    );

    let mut context = test.start_with_context().await;
    let deposit_token_mint = Keypair::new();
    let payer = &context.payer.pubkey();
    create_mint(&mut context, &deposit_token_mint, payer)
        .await
        .unwrap();

    let test_reward_pool = TestRewards::new(deposit_token_mint.pubkey());

    test_reward_pool
        .initialize_pool(&mut context)
        .await
        .unwrap();

    let mining_owner = Keypair::new();
    let user_mining = test_reward_pool
        .initialize_mining(&mut context, &mining_owner.pubkey())
        .await;

    (context, test_reward_pool, mining_owner, user_mining)
}

#[tokio::test]
async fn success() {
    let (mut context, test_rewards, mining_owner, mining) = setup().await;
    let mining_owner_before = context
        .banks_client
        .get_account(mining_owner.pubkey())
        .await
        .unwrap();
    assert_eq!(None, mining_owner_before);

    test_rewards
        .deposit_mining(
            &mut context,
            &mining,
            100,
            LockupPeriod::ThreeMonths,
            &mining_owner.pubkey(),
        )
        .await
        .unwrap();

    test_rewards
        .close_mining(&mut context, &mining, &mining_owner, &mining_owner.pubkey())
        .await
        .unwrap();

    let mining_account_after = context.banks_client.get_account(mining).await.unwrap();
    assert_eq!(None, mining_account_after);

    let mining_owner = get_account(&mut context, &mining_owner.pubkey()).await;
    assert!(mining_owner.lamports > 0);
}

#[tokio::test]
async fn success_after_not_interacting_for_a_long_time() {
    let (mut context, test_rewards, mining_owner, mining) = setup().await;
    let mining_owner_before = context
        .banks_client
        .get_account(mining_owner.pubkey())
        .await
        .unwrap();
    assert_eq!(None, mining_owner_before);

    test_rewards
        .deposit_mining(
            &mut context,
            &mining,
            100,
            LockupPeriod::ThreeMonths,
            &mining_owner.pubkey(),
        )
        .await
        .unwrap();

    // fill vault with tokens
    let distribution_ends_at = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64
        + SECONDS_PER_DAY * 100;

    // mint token for fill_authority aka wallet who will fill the vault with tokens
    let rewarder = Keypair::new();
    create_token_account(
        &mut context,
        &rewarder,
        &test_rewards.token_mint_pubkey,
        &test_rewards.fill_authority.pubkey(),
        0,
    )
    .await
    .unwrap();
    mint_tokens(
        &mut context,
        &test_rewards.token_mint_pubkey,
        &rewarder.pubkey(),
        100,
    )
    .await
    .unwrap();

    test_rewards
        .fill_vault(&mut context, &rewarder.pubkey(), 100, distribution_ends_at)
        .await
        .unwrap();
    // distribute rewards to users
    test_rewards.distribute_rewards(&mut context).await.unwrap();

    advance_clock_by_ts(&mut context, (SECONDS_PER_DAY * 100).try_into().unwrap()).await;

    test_rewards
        .close_mining(&mut context, &mining, &mining_owner, &mining_owner.pubkey())
        .await
        .assert_on_chain_err(MplxRewardsError::RewardsMustBeClaimed);
}
