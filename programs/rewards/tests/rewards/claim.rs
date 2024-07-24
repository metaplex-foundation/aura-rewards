use crate::utils::*;
use mplx_rewards::{
    state::{RewardPool, WrappedMining},
    utils::LockupPeriod,
};
use solana_program::{program_pack::Pack, pubkey::Pubkey};
use solana_program_test::*;
use solana_sdk::{clock::SECONDS_PER_DAY, signature::Keypair, signer::Signer};
use spl_token::state::Account;
use std::borrow::{Borrow, BorrowMut};

async fn setup() -> (ProgramTestContext, TestRewards, Pubkey) {
    let test = ProgramTest::new(
        "mplx_rewards",
        mplx_rewards::id(),
        processor!(mplx_rewards::processor::process_instruction),
    );
    let mut context = test.start_with_context().await;

    let owner = &context.payer.pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, owner).await.unwrap();

    let test_rewards = TestRewards::new(mint.pubkey());
    test_rewards.initialize_pool(&mut context).await.unwrap();

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
        1_000_000,
    )
    .await
    .unwrap();

    (context, test_rewards, rewarder.pubkey())
}

#[tokio::test]
async fn with_two_users() {
    let (mut context, test_rewards, rewarder) = setup().await;

    let (user_a, user_rewards_a, user_mining_a) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_a,
            100,
            LockupPeriod::ThreeMonths,
            &user_a.pubkey(),
            &user_mining_a,
        )
        .await
        .unwrap();

    let (user_b, user_rewards_b, user_mining_b) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_b,
            100,
            LockupPeriod::ThreeMonths,
            &user_b.pubkey(),
            &user_mining_b,
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
        + SECONDS_PER_DAY;

    test_rewards
        .fill_vault(&mut context, &rewarder, 100, distribution_ends_at)
        .await
        .unwrap();

    test_rewards.distribute_rewards(&mut context).await.unwrap();

    test_rewards
        .claim(
            &mut context,
            &user_a,
            &user_mining_a,
            &user_rewards_a.pubkey(),
        )
        .await
        .unwrap();

    test_rewards
        .claim(
            &mut context,
            &user_b,
            &user_mining_b,
            &user_rewards_b.pubkey(),
        )
        .await
        .unwrap();

    let user_reward_account_a = get_account(&mut context, &user_rewards_a.pubkey()).await;
    let user_rewards_a = Account::unpack(user_reward_account_a.data.borrow()).unwrap();

    assert_eq!(user_rewards_a.amount, 50);

    let user_reward_account_b = get_account(&mut context, &user_rewards_b.pubkey()).await;
    let user_rewards_b = Account::unpack(user_reward_account_b.data.borrow()).unwrap();

    assert_eq!(user_rewards_b.amount, 50);
}

#[tokio::test]
async fn flex_vs_three_months() {
    let (mut context, test_rewards, rewarder) = setup().await;

    let (user_a, user_rewards_a, user_mining_a) =
        create_end_user(&mut context, &test_rewards).await;

    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_a,
            100,
            LockupPeriod::ThreeMonths,
            &user_a.pubkey(),
            &user_mining_a,
        )
        .await
        .unwrap();
    // warp to three month ahead
    advance_clock_by_ts(&mut context, (SECONDS_PER_DAY * 91).try_into().unwrap()).await;

    let (user_b, user_rewards_b, user_mining_b) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_b,
            100,
            LockupPeriod::ThreeMonths,
            &user_b.pubkey(),
            &user_mining_b,
        )
        .await
        .unwrap();

    let distribution_ends_at = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64
        + SECONDS_PER_DAY;

    test_rewards
        .fill_vault(&mut context, &rewarder, 1_000, distribution_ends_at)
        .await
        .unwrap();

    test_rewards.distribute_rewards(&mut context).await.unwrap();

    test_rewards
        .claim(
            &mut context,
            &user_a,
            &user_mining_a,
            &user_rewards_a.pubkey(),
        )
        .await
        .unwrap();

    let user_reward_account_a = get_account(&mut context, &user_rewards_a.pubkey()).await;
    let user_rewards_a = Account::unpack(user_reward_account_a.data.borrow()).unwrap();

    assert_eq!(user_rewards_a.amount, 333);

    test_rewards
        .claim(
            &mut context,
            &user_b,
            &user_mining_b,
            &user_rewards_b.pubkey(),
        )
        .await
        .unwrap();

    let user_reward_account_b = get_account(&mut context, &user_rewards_b.pubkey()).await;
    let user_rewards_b = Account::unpack(user_reward_account_b.data.borrow()).unwrap();

    assert_eq!(user_rewards_b.amount, 666);
}

#[tokio::test]
// User 1: lockup for ThreeMonth, 5 distributions, 1 claim
// User 2: lockup for OneYear, 5 distributions, 5 claims
async fn multiple_consequantial_distributions_for_two_users() {
    let (mut context, test_rewards, rewarder) = setup().await;

    let (user_a, user_rewards_a, user_mining_a) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_a,
            100,
            LockupPeriod::ThreeMonths,
            &user_a.pubkey(),
            &user_mining_a,
        )
        .await
        .unwrap();

    let (user_b, user_rewards_b, user_mining_b) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_b,
            100,
            LockupPeriod::OneYear,
            &user_b.pubkey(),
            &user_mining_b,
        )
        .await
        .unwrap();

    let distribution_ends_at = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64
        + SECONDS_PER_DAY * 6;

    test_rewards
        .fill_vault(&mut context, &rewarder, 500, distribution_ends_at)
        .await
        .unwrap();

    // 5 days of daily reward claiming for user2
    for _ in 0..5 {
        test_rewards.distribute_rewards(&mut context).await.unwrap();

        test_rewards
            .claim(
                &mut context,
                &user_b,
                &user_mining_b,
                &user_rewards_b.pubkey(),
            )
            .await
            .unwrap();

        advance_clock_by_ts(&mut context, SECONDS_PER_DAY.try_into().unwrap()).await;
    }

    test_rewards
        .claim(
            &mut context,
            &user_b,
            &user_mining_b,
            &user_rewards_b.pubkey(),
        )
        .await
        .unwrap();

    test_rewards
        .claim(
            &mut context,
            &user_a,
            &user_mining_a,
            &user_rewards_a.pubkey(),
        )
        .await
        .unwrap();

    let user_reward_account_a = get_account(&mut context, &user_rewards_a.pubkey()).await;
    let user_rewards_a = Account::unpack(user_reward_account_a.data.borrow()).unwrap();

    assert_eq!(user_rewards_a.amount, 125);

    let user_reward_account_b = get_account(&mut context, &user_rewards_b.pubkey()).await;
    let user_rewards_b = Account::unpack(user_reward_account_b.data.borrow()).unwrap();

    assert_eq!(user_rewards_b.amount, 375);
}

#[tokio::test]
// User 1: lockup for ThreeMonth, 5 distributions, 1 claim
// User 2: lockup for OneYear, 5 distributions, 5 claims
async fn rewards_after_distribution_are_unclaimable() {
    let (mut context, test_rewards, rewarder) = setup().await;

    let (user_a, user_rewards_a, user_mining_a) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_a,
            100,
            LockupPeriod::ThreeMonths,
            &user_a.pubkey(),
            &user_mining_a,
        )
        .await
        .unwrap();

    let distribution_ends_at = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64
        + SECONDS_PER_DAY;

    test_rewards
        .fill_vault(&mut context, &rewarder, 1000, distribution_ends_at)
        .await
        .unwrap();
    test_rewards.distribute_rewards(&mut context).await.unwrap();

    test_rewards
        .claim(
            &mut context,
            &user_a,
            &user_mining_a,
            &user_rewards_a.pubkey(),
        )
        .await
        .unwrap();

    let user_reward_account = get_account(&mut context, &user_rewards_a.pubkey()).await;
    let user_reward = Account::unpack(user_reward_account.data.borrow()).unwrap();
    assert_eq!(user_reward.amount, 1_000);

    advance_clock_by_ts(&mut context, (SECONDS_PER_DAY * 1000).try_into().unwrap()).await;
    let distribution_ends_at = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64
        + SECONDS_PER_DAY;
    test_rewards
        .fill_vault(&mut context, &rewarder, 100, distribution_ends_at)
        .await
        .unwrap();

    let (user_b, user_rewards_b, user_mining_b) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_b,
            100,
            LockupPeriod::OneYear,
            &user_b.pubkey(),
            &user_mining_b,
        )
        .await
        .unwrap();

    test_rewards
        .claim(
            &mut context,
            &user_b,
            &user_mining_b,
            &user_rewards_b.pubkey(),
        )
        .await
        .unwrap();

    let user_reward_account2 = get_account(&mut context, &user_rewards_b.pubkey()).await;
    let user_reward2 = Account::unpack(user_reward_account2.data.borrow()).unwrap();

    assert_eq!(user_reward2.amount, 0);
}

#[tokio::test]
async fn switch_to_flex_is_correct() {
    let (mut context, test_rewards, rewarder) = setup().await;

    let (user_a, user_rewards_a, user_mining_a) =
        create_end_user(&mut context, &test_rewards).await;

    // D1
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_a,
            100,
            LockupPeriod::ThreeMonths,
            &user_a.pubkey(),
            &user_mining_a,
        )
        .await
        .unwrap();

    let (user_b, user_rewards_b, user_mining_b) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_b,
            100,
            LockupPeriod::OneYear,
            &user_b.pubkey(),
            &user_mining_b,
        )
        .await
        .unwrap();

    // warp to day 91 to expire the deposit D1
    advance_clock_by_ts(&mut context, (SECONDS_PER_DAY * 91).try_into().unwrap()).await;

    let distribution_ends_at = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64
        + SECONDS_PER_DAY;
    test_rewards
        .fill_vault(&mut context, &rewarder, 100, distribution_ends_at)
        .await
        .unwrap();
    test_rewards.distribute_rewards(&mut context).await.unwrap();

    test_rewards
        .claim(
            &mut context,
            &user_a,
            &user_mining_a,
            &user_rewards_a.pubkey(),
        )
        .await
        .unwrap();

    test_rewards
        .claim(
            &mut context,
            &user_b,
            &user_mining_b,
            &user_rewards_b.pubkey(),
        )
        .await
        .unwrap();

    let user_reward_account_a = get_account(&mut context, &user_rewards_a.pubkey()).await;
    let user_rewards_a = Account::unpack(user_reward_account_a.data.borrow()).unwrap();

    assert_eq!(user_rewards_a.amount, 14);

    let user_reward_account_b = get_account(&mut context, &user_rewards_b.pubkey()).await;
    let user_rewards_b = Account::unpack(user_reward_account_b.data.borrow()).unwrap();

    assert_eq!(user_rewards_b.amount, 85);
}

#[tokio::test]
async fn two_deposits_vs_one() {
    let (mut context, test_rewards, rewarder) = setup().await;

    let (user_a, user_rewards_a, user_mining_a) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_a,
            100,
            LockupPeriod::OneYear,
            &user_a.pubkey(),
            &user_mining_a,
        )
        .await
        .unwrap();

    let (user_b, user_rewards_b, user_mining_b) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_b,
            50,
            LockupPeriod::OneYear,
            &user_b.pubkey(),
            &user_mining_b,
        )
        .await
        .unwrap();
    // AVOID CACHING FOR IDENTICAL OPERATIONS
    let initial_slot = context.banks_client.get_root_slot().await.unwrap();
    context.warp_to_slot(initial_slot + 1).unwrap();
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_b,
            50,
            LockupPeriod::OneYear,
            &user_b.pubkey(),
            &user_mining_b,
        )
        .await
        .unwrap();

    let distribution_ends_at = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64
        + SECONDS_PER_DAY;
    test_rewards
        .fill_vault(&mut context, &rewarder, 1000, distribution_ends_at)
        .await
        .unwrap();
    test_rewards.distribute_rewards(&mut context).await.unwrap();

    test_rewards
        .claim(
            &mut context,
            &user_a,
            &user_mining_a,
            &user_rewards_a.pubkey(),
        )
        .await
        .unwrap();

    test_rewards
        .claim(
            &mut context,
            &user_b,
            &user_mining_b,
            &user_rewards_b.pubkey(),
        )
        .await
        .unwrap();

    let user_reward_account_a = get_account(&mut context, &user_rewards_a.pubkey()).await;
    let user_rewards_a = Account::unpack(user_reward_account_a.data.borrow()).unwrap();

    assert_eq!(user_rewards_a.amount, 499);

    let user_reward_account_b = get_account(&mut context, &user_rewards_b.pubkey()).await;
    let user_rewards_b = Account::unpack(user_reward_account_b.data.borrow()).unwrap();

    assert_eq!(user_rewards_b.amount, 499);
}

#[tokio::test]
async fn claim_tokens_after_deposit_expiration() {
    let (mut context, test_rewards, rewarder) = setup().await;

    let (user_a, user_rewards_a, user_mining_a) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_a,
            100,
            LockupPeriod::OneYear,
            &user_a.pubkey(),
            &user_mining_a,
        )
        .await
        .unwrap();

    let (user_b, user_rewards_b, user_mining_b) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_b,
            300,
            LockupPeriod::ThreeMonths,
            &user_b.pubkey(),
            &user_mining_b,
        )
        .await
        .unwrap();

    let distribution_ends_at = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64
        + SECONDS_PER_DAY;
    test_rewards
        .fill_vault(&mut context, &rewarder, 1000, distribution_ends_at)
        .await
        .unwrap();
    test_rewards.distribute_rewards(&mut context).await.unwrap();

    advance_clock_by_ts(&mut context, (180 * SECONDS_PER_DAY).try_into().unwrap()).await;

    test_rewards
        .claim(
            &mut context,
            &user_a,
            &user_mining_a,
            &user_rewards_a.pubkey(),
        )
        .await
        .unwrap();

    test_rewards
        .claim(
            &mut context,
            &user_b,
            &user_mining_b,
            &user_rewards_b.pubkey(),
        )
        .await
        .unwrap();

    let user_reward_account_a = get_account(&mut context, &user_rewards_a.pubkey()).await;
    let user_rewards_a = Account::unpack(user_reward_account_a.data.borrow()).unwrap();

    assert_eq!(user_rewards_a.amount, 499);

    let user_reward_account_b = get_account(&mut context, &user_rewards_b.pubkey()).await;
    let user_rewards_b = Account::unpack(user_reward_account_b.data.borrow()).unwrap();

    assert_eq!(user_rewards_b.amount, 499);
}

#[tokio::test]
async fn claim_after_withdraw_is_correct() {
    let (mut context, test_rewards, rewarder) = setup().await;

    let (user_a, user_rewards_a, user_mining_a) =
        create_end_user(&mut context, &test_rewards).await;

    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_a,
            100,
            LockupPeriod::OneYear,
            &user_a.pubkey(),
            &user_mining_a,
        )
        .await
        .unwrap();
    let (user_b, user_rewards_b, user_mining_b) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_b,
            50,
            LockupPeriod::OneYear,
            &user_b.pubkey(),
            &user_mining_b,
        )
        .await
        .unwrap();
    // D3
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_b,
            150,
            LockupPeriod::ThreeMonths,
            &user_b.pubkey(),
            &user_mining_b,
        )
        .await
        .unwrap();

    let distribution_ends_at = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64
        + SECONDS_PER_DAY;
    test_rewards
        .fill_vault(&mut context, &rewarder, 100, distribution_ends_at)
        .await
        .unwrap();
    test_rewards.distribute_rewards(&mut context).await.unwrap();

    claim_and_assert(
        &test_rewards,
        &mut context,
        &user_a,
        &user_mining_a,
        &user_rewards_a.pubkey(),
        49,
    )
    .await;
    claim_and_assert(
        &test_rewards,
        &mut context,
        &user_b,
        &user_mining_b,
        &user_rewards_b.pubkey(),
        49,
    )
    .await;

    // T = 1200, A = 600, B = 300 + 300

    // warp to three month ahead to expire D3
    advance_clock_by_ts(&mut context, (SECONDS_PER_DAY * 91).try_into().unwrap()).await;
    let distribution_ends_at: u64 = (context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64)
        .checked_add(SECONDS_PER_DAY)
        .unwrap();

    test_rewards
        .fill_vault(&mut context, &rewarder, 100, distribution_ends_at)
        .await
        .unwrap();
    test_rewards.distribute_rewards(&mut context).await.unwrap();

    // T = 1050, A = 600, B = 300 + 150
    claim_and_assert(
        &test_rewards,
        &mut context,
        &user_a,
        &user_mining_a,
        &user_rewards_a.pubkey(),
        49 + 57,
    )
    .await;
    claim_and_assert(
        &test_rewards,
        &mut context,
        &user_b,
        &user_mining_b,
        &user_rewards_b.pubkey(),
        49 + 42,
    )
    .await;

    test_rewards
        .withdraw_mining(
            &mut context,
            &user_mining_b,
            &user_mining_b,
            150,
            &user_b.pubkey(),
        )
        .await
        .unwrap();

    advance_clock_by_ts(&mut context, SECONDS_PER_DAY.try_into().unwrap()).await;
    let distribution_ends_at = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64
        + SECONDS_PER_DAY;
    test_rewards
        .fill_vault(&mut context, &rewarder, 100, distribution_ends_at)
        .await
        .unwrap();
    test_rewards.distribute_rewards(&mut context).await.unwrap();

    claim_and_assert(
        &test_rewards,
        &mut context,
        &user_a,
        &user_mining_a,
        &user_rewards_a.pubkey(),
        172,
    )
    .await;
    claim_and_assert(
        &test_rewards,
        &mut context,
        &user_b,
        &user_mining_b,
        &user_rewards_b.pubkey(),
        124,
    )
    .await;
}

#[tokio::test]
async fn with_two_users_with_flex() {
    let (mut context, test_rewards, rewarder) = setup().await;

    let (user_a, user_rewards_a, user_mining_a) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_a,
            100,
            LockupPeriod::Flex,
            &user_a.pubkey(),
            &user_mining_a,
        )
        .await
        .unwrap();

    let (user_b, user_rewards_b, user_mining_b) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_b,
            100,
            LockupPeriod::Flex,
            &user_b.pubkey(),
            &user_mining_b,
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
        + SECONDS_PER_DAY;

    test_rewards
        .fill_vault(&mut context, &rewarder, 100, distribution_ends_at)
        .await
        .unwrap();

    test_rewards.distribute_rewards(&mut context).await.unwrap();

    test_rewards
        .claim(
            &mut context,
            &user_a,
            &user_mining_a,
            &user_rewards_a.pubkey(),
        )
        .await
        .unwrap();

    test_rewards
        .claim(
            &mut context,
            &user_b,
            &user_mining_b,
            &user_rewards_b.pubkey(),
        )
        .await
        .unwrap();

    let user_reward_account_a = get_account(&mut context, &user_rewards_a.pubkey()).await;
    let user_rewards_a = Account::unpack(user_reward_account_a.data.borrow()).unwrap();

    assert_eq!(user_rewards_a.amount, 50);

    let user_reward_account_b = get_account(&mut context, &user_rewards_b.pubkey()).await;
    let user_rewards_b = Account::unpack(user_reward_account_b.data.borrow()).unwrap();

    assert_eq!(user_rewards_b.amount, 50);
}

#[tokio::test]
async fn claim_with_delegate() {
    let (mut context, test_rewards, rewarder) = setup().await;

    let (delegate, delegate_rewards, delegate_mining) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &delegate_mining,
            3_000_000, // 18_000_000 of weighted stake
            LockupPeriod::OneYear,
            &delegate.pubkey(),
            &delegate_mining,
        )
        .await
        .unwrap();
    let mut delegate_mining_account = get_account(&mut context, &delegate_mining).await;
    let d_mining_data = &mut delegate_mining_account.data.borrow_mut();
    let d_wrapped_mining = WrappedMining::from_bytes_mut(d_mining_data).unwrap();
    assert_eq!(d_wrapped_mining.mining.share, 18_000_000);
    assert_eq!(d_wrapped_mining.mining.stake_from_others, 0);

    let (user_a, user_rewards_a, user_mining_a) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_a,
            1_000_000, //  6_000_000 of weighted stake
            LockupPeriod::OneYear,
            &user_a.pubkey(),
            &delegate_mining,
        )
        .await
        .unwrap();

    let mut delegate_mining_account = get_account(&mut context, &delegate_mining).await;
    let d_mining_data = &mut delegate_mining_account.data.borrow_mut();
    let d_wrapped_mining = WrappedMining::from_bytes_mut(d_mining_data).unwrap();
    assert_eq!(d_wrapped_mining.mining.share, 18_000_000);
    assert_eq!(d_wrapped_mining.mining.stake_from_others, 1_000_000);

    let reward_pool_account = get_account(&mut context, &test_rewards.reward_pool).await;
    let reward_pool = RewardPool::unpack(reward_pool_account.data.borrow()).unwrap();

    assert_eq!(reward_pool.total_share, 25_000_000);

    let mut mining_account = get_account(&mut context, &user_mining_a).await;
    let mining_data = &mut mining_account.data.borrow_mut();
    let wrapped_mining = WrappedMining::from_bytes_mut(mining_data).unwrap();
    assert_eq!(wrapped_mining.mining.share, 6_000_000);

    // fill vault with tokens
    let distribution_ends_at = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64;

    test_rewards
        .fill_vault(&mut context, &rewarder, 1_000_000, distribution_ends_at)
        .await
        .unwrap();
    // distribute rewards to users
    test_rewards.distribute_rewards(&mut context).await.unwrap();

    test_rewards
        .claim(
            &mut context,
            &user_a,
            &user_mining_a,
            &user_rewards_a.pubkey(),
        )
        .await
        .unwrap();
    test_rewards
        .claim(
            &mut context,
            &delegate,
            &delegate_mining,
            &delegate_rewards.pubkey(),
        )
        .await
        .unwrap();

    let user_reward_account_a = get_account(&mut context, &user_rewards_a.pubkey()).await;
    let user_rewards_a = Account::unpack(user_reward_account_a.data.borrow()).unwrap();

    assert_eq!(user_rewards_a.amount, 240_000);

    let delegate_account = get_account(&mut context, &delegate_rewards.pubkey()).await;
    let delegate_rewards = Account::unpack(delegate_account.data.borrow()).unwrap();

    assert_eq!(delegate_rewards.amount, 760_000);
}
