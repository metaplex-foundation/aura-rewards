use crate::utils::*;
use mplx_rewards::utils::LockupPeriod;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::clock::SECONDS_PER_DAY;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use spl_token::state::Account;
use std::borrow::Borrow;

async fn setup() -> (ProgramTestContext, TestRewards, Pubkey, Keypair) {
    let (mut context, _) = presetup().await;

    let owner = &context.payer.pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, owner).await.unwrap();

    let test_reward_pool = TestRewards::new(Some(mint.pubkey()));
    test_reward_pool
        .initialize_pool(&mut context)
        .await
        .unwrap();

    let rewarder = Keypair::new();
    create_token_account(&mut context, &rewarder, &mint.pubkey(), owner, 0)
        .await
        .unwrap();
    mint_tokens(&mut context, &mint.pubkey(), &rewarder.pubkey(), 1_000_000)
        .await
        .unwrap();

    test_reward_pool.add_vault(&mut context).await;

    (context, test_reward_pool, rewarder.pubkey(), mint)
}

#[tokio::test]
async fn success() {
    let (mut context, test_rewards_pool, rewarder, mint) = setup().await;

    let (user, user_rewards, user_mining) = create_user(&mut context, &test_rewards_pool).await;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user.pubkey(),
            &user_mining,
            100,
            LockupPeriod::ThreeMonths,
            &mint.pubkey(),
        )
        .await
        .unwrap();

    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 1_000_000)
        .await
        .unwrap();

    test_rewards_pool
        .claim(&mut context, &user, &user_mining, &user_rewards.pubkey())
        .await
        .unwrap();

    let user_reward_account = get_account(&mut context, &user_rewards.pubkey()).await;
    let user_reward = Account::unpack(user_reward_account.data.borrow()).unwrap();

    assert_eq!(user_reward.amount, 1_000_000);
}

#[tokio::test]
async fn with_two_users() {
    let (mut context, test_rewards_pool, rewarder, mint) = setup().await;

    let (user_a, user_rewards_a, user_mining_a) =
        create_user(&mut context, &test_rewards_pool).await;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_a.pubkey(),
            &user_mining_a,
            100,
            LockupPeriod::ThreeMonths,
            &mint.pubkey(),
        )
        .await
        .unwrap();

    let (user_b, user_rewards_b, user_mining_b) =
        create_user(&mut context, &test_rewards_pool).await;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_b.pubkey(),
            &user_mining_b,
            100,
            LockupPeriod::ThreeMonths,
            &test_rewards_pool.token_mint_pubkey,
        )
        .await
        .unwrap();

    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 1_000_000)
        .await
        .unwrap();

    test_rewards_pool
        .claim(
            &mut context,
            &user_a,
            &user_mining_a,
            &user_rewards_a.pubkey(),
        )
        .await
        .unwrap();

    test_rewards_pool
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

    assert_eq!(user_rewards_a.amount, 500_000);

    let user_reward_account_b = get_account(&mut context, &user_rewards_b.pubkey()).await;
    let user_rewards_b = Account::unpack(user_reward_account_b.data.borrow()).unwrap();

    assert_eq!(user_rewards_b.amount, 500_000);
}

#[tokio::test]
async fn flex_vs_three_months() {
    let (mut context, test_rewards_pool, rewarder, mint) = setup().await;

    let (user_a, user_rewards_a, user_mining_a) =
        create_user(&mut context, &test_rewards_pool).await;

    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_a.pubkey(),
            &user_mining_a,
            100,
            LockupPeriod::ThreeMonths,
            &mint.pubkey(),
        )
        .await
        .unwrap();
    // warp to three month ahead
    advance_clock_by_ts(&mut context, (SECONDS_PER_DAY * 91).try_into().unwrap()).await;

    let (user_b, user_rewards_b, user_mining_b) =
        create_user(&mut context, &test_rewards_pool).await;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_b.pubkey(),
            &user_mining_b,
            100,
            LockupPeriod::ThreeMonths,
            &test_rewards_pool.token_mint_pubkey,
        )
        .await
        .unwrap();

    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 1_000)
        .await
        .unwrap();

    test_rewards_pool
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

    test_rewards_pool
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
async fn multiple_consequantial_distributions_for_two_user() {
    let (mut context, test_rewards_pool, rewarder, mint) = setup().await;

    let (user_a, user_rewards_a, user_mining_a) =
        create_user(&mut context, &test_rewards_pool).await;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_a.pubkey(),
            &user_mining_a,
            100,
            LockupPeriod::ThreeMonths,
            &mint.pubkey(),
        )
        .await
        .unwrap();

    let (user_b, user_rewards_b, user_mining_b) =
        create_user(&mut context, &test_rewards_pool).await;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_b.pubkey(),
            &user_mining_b,
            100,
            LockupPeriod::OneYear,
            &test_rewards_pool.token_mint_pubkey,
        )
        .await
        .unwrap();

    // 4 days of daily reward claiming for user2
    for _ in 0..4 {
        test_rewards_pool
            .fill_vault(&mut context, &rewarder, 100)
            .await
            .unwrap();

        test_rewards_pool
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

    // day 5. User2 and User1 claim
    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();

    test_rewards_pool
        .claim(
            &mut context,
            &user_b,
            &user_mining_b,
            &user_rewards_b.pubkey(),
        )
        .await
        .unwrap();

    test_rewards_pool
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
    let (mut context, test_rewards_pool, rewarder, mint) = setup().await;

    let (user_a, user_rewards_a, user_mining_a) =
        create_user(&mut context, &test_rewards_pool).await;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_a.pubkey(),
            &user_mining_a,
            100,
            LockupPeriod::ThreeMonths,
            &mint.pubkey(),
        )
        .await
        .unwrap();

    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 1_000)
        .await
        .unwrap();

    test_rewards_pool
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
    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();

    let (user_b, user_rewards_b, user_mining_b) =
        create_user(&mut context, &test_rewards_pool).await;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_b.pubkey(),
            &user_mining_b,
            100,
            LockupPeriod::OneYear,
            &test_rewards_pool.token_mint_pubkey,
        )
        .await
        .unwrap();

    test_rewards_pool
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
    let (mut context, test_rewards_pool, rewarder, mint) = setup().await;

    let (user_a, user_rewards_a, user_mining_a) =
        create_user(&mut context, &test_rewards_pool).await;

    // D1
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_a.pubkey(),
            &user_mining_a,
            100,
            LockupPeriod::ThreeMonths,
            &mint.pubkey(),
        )
        .await
        .unwrap();

    let (user_b, user_rewards_b, user_mining_b) =
        create_user(&mut context, &test_rewards_pool).await;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_b.pubkey(),
            &user_mining_b,
            100,
            LockupPeriod::OneYear,
            &test_rewards_pool.token_mint_pubkey,
        )
        .await
        .unwrap();

    // warp to day 91 to expire the deposit D1
    advance_clock_by_ts(&mut context, (SECONDS_PER_DAY * 91).try_into().unwrap()).await;

    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();

    test_rewards_pool
        .claim(
            &mut context,
            &user_a,
            &user_mining_a,
            &user_rewards_a.pubkey(),
        )
        .await
        .unwrap();

    test_rewards_pool
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
    let (mut context, test_rewards_pool, rewarder, mint) = setup().await;

    let (user_a, user_rewards_a, user_mining_a) =
        create_user(&mut context, &test_rewards_pool).await;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_a.pubkey(),
            &user_mining_a,
            100,
            LockupPeriod::OneYear,
            &mint.pubkey(),
        )
        .await
        .unwrap();

    let (user_b, user_rewards_b, user_mining_b) =
        create_user(&mut context, &test_rewards_pool).await;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_b.pubkey(),
            &user_mining_b,
            50,
            LockupPeriod::OneYear,
            &test_rewards_pool.token_mint_pubkey,
        )
        .await
        .unwrap();
    // AVOID CACHING FOR IDENTICAL OPERATIONS
    let initial_slot = context.banks_client.get_root_slot().await.unwrap();
    context.warp_to_slot(initial_slot + 1).unwrap();
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_b.pubkey(),
            &user_mining_b,
            50,
            LockupPeriod::OneYear,
            &test_rewards_pool.token_mint_pubkey,
        )
        .await
        .unwrap();

    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 1000)
        .await
        .unwrap();

    test_rewards_pool
        .claim(
            &mut context,
            &user_a,
            &user_mining_a,
            &user_rewards_a.pubkey(),
        )
        .await
        .unwrap();

    test_rewards_pool
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
    let (mut context, test_rewards_pool, rewarder, mint) = setup().await;

    let (user_a, user_rewards_a, user_mining_a) =
        create_user(&mut context, &test_rewards_pool).await;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_a.pubkey(),
            &user_mining_a,
            100,
            LockupPeriod::OneYear,
            &mint.pubkey(),
        )
        .await
        .unwrap();

    let (user_b, user_rewards_b, user_mining_b) =
        create_user(&mut context, &test_rewards_pool).await;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_b.pubkey(),
            &user_mining_b,
            300,
            LockupPeriod::ThreeMonths,
            &test_rewards_pool.token_mint_pubkey,
        )
        .await
        .unwrap();

    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 1000)
        .await
        .unwrap();

    advance_clock_by_ts(&mut context, (180 * SECONDS_PER_DAY).try_into().unwrap()).await;

    test_rewards_pool
        .claim(
            &mut context,
            &user_a,
            &user_mining_a,
            &user_rewards_a.pubkey(),
        )
        .await
        .unwrap();

    test_rewards_pool
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
    let (mut context, test_rewards_pool, rewarder, mint) = setup().await;

    let (user_a, user_rewards_a, user_mining_a) =
        create_user(&mut context, &test_rewards_pool).await;

    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_a.pubkey(),
            &user_mining_a,
            100,
            LockupPeriod::OneYear,
            &mint.pubkey(),
        )
        .await
        .unwrap();
    let (user_b, user_rewards_b, user_mining_b) =
        create_user(&mut context, &test_rewards_pool).await;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_b.pubkey(),
            &user_mining_b,
            50,
            LockupPeriod::OneYear,
            &test_rewards_pool.token_mint_pubkey,
        )
        .await
        .unwrap();
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_b.pubkey(),
            &user_mining_b,
            150,
            LockupPeriod::ThreeMonths,
            &test_rewards_pool.token_mint_pubkey,
        )
        .await
        .unwrap();

    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();

    // T = 1200, A = 600, B = 300 + 300

    // warp to three month ahead
    advance_clock_by_ts(&mut context, (SECONDS_PER_DAY * 91).try_into().unwrap()).await;

    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();

    test_rewards_pool
        .withdraw_mining(&mut context, &user_b.pubkey(), &user_mining_b, 150)
        .await
        .unwrap();

    advance_clock_by_ts(&mut context, SECONDS_PER_DAY.try_into().unwrap()).await;
    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();

    claim_and_assert(
        &test_rewards_pool,
        &mut context,
        &user_a,
        &user_mining_a,
        &user_rewards_a.pubkey(),
        173,
    )
    .await;
    claim_and_assert(
        &test_rewards_pool,
        &mut context,
        &user_b,
        &user_mining_b,
        &user_rewards_b.pubkey(),
        124,
    )
    .await;
}
