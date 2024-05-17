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

async fn setup() -> (ProgramTestContext, TestRewards, Keypair, Pubkey, Pubkey) {
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

    // TODO: extract user func
    let user = Keypair::new();
    let account = Keypair::new();
    create_token_account(
        &mut context,
        &account,
        &test_reward_pool.token_mint_pubkey,
        &user.pubkey(),
        0,
    )
    .await
    .unwrap();
    let user_mining = test_reward_pool
        .initialize_mining(&mut context, &user.pubkey())
        .await;
    let lockup_period = LockupPeriod::ThreeMonths;
    test_reward_pool
        .deposit_mining(
            &mut context,
            &user.pubkey(),
            &user_mining,
            100,
            lockup_period,
            &mint.pubkey(),
        )
        .await
        .unwrap();

    (
        context,
        test_reward_pool,
        user,
        user_mining,
        rewarder.pubkey(),
    )
}

#[tokio::test]
async fn success() {
    let (mut context, test_rewards_pool, user, user_mining, rewarder) = setup().await;

    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 1_000_000)
        .await
        .unwrap();

    let user_reward = Keypair::new();
    create_token_account(
        &mut context,
        &user_reward,
        &test_rewards_pool.token_mint_pubkey,
        &user.pubkey(),
        0,
    )
    .await
    .unwrap();

    test_rewards_pool
        .claim(&mut context, &user, &user_mining, &user_reward.pubkey())
        .await
        .unwrap();

    let user_reward_account = get_account(&mut context, &user_reward.pubkey()).await;
    let user_reward = Account::unpack(user_reward_account.data.borrow()).unwrap();

    assert_eq!(user_reward.amount, 1_000_000);
}

#[tokio::test]
async fn with_two_users() {
    let (mut context, test_rewards_pool, user1, user_mining1, rewarder) = setup().await;

    let user2 = Keypair::new();
    let user_mining2 = test_rewards_pool
        .initialize_mining(&mut context, &user2.pubkey())
        .await;
    let lockup_period = LockupPeriod::ThreeMonths;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user2.pubkey(),
            &user_mining2,
            100,
            lockup_period,
            &test_rewards_pool.token_mint_pubkey,
        )
        .await
        .unwrap();

    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 1_000_000)
        .await
        .unwrap();

    let user_reward1 = Keypair::new();
    create_token_account(
        &mut context,
        &user_reward1,
        &test_rewards_pool.token_mint_pubkey,
        &user1.pubkey(),
        0,
    )
    .await
    .unwrap();

    test_rewards_pool
        .claim(&mut context, &user1, &user_mining1, &user_reward1.pubkey())
        .await
        .unwrap();

    let user_reward2 = Keypair::new();
    create_token_account(
        &mut context,
        &user_reward2,
        &test_rewards_pool.token_mint_pubkey,
        &user2.pubkey(),
        0,
    )
    .await
    .unwrap();

    test_rewards_pool
        .claim(&mut context, &user2, &user_mining2, &user_reward2.pubkey())
        .await
        .unwrap();

    let user_reward_account1 = get_account(&mut context, &user_reward1.pubkey()).await;
    let user_reward1 = Account::unpack(user_reward_account1.data.borrow()).unwrap();

    assert_eq!(user_reward1.amount, 500_000);

    let user_reward_account2 = get_account(&mut context, &user_reward2.pubkey()).await;
    let user_reward2 = Account::unpack(user_reward_account2.data.borrow()).unwrap();

    assert_eq!(user_reward2.amount, 500_000);
}

#[tokio::test]
async fn flex_vs_three_months() {
    let (mut context, test_rewards_pool, user1, user_mining1, rewarder) = setup().await;
    // warp to three month ahead
    advance_clock_by_ts(&mut context, (SECONDS_PER_DAY * 90).try_into().unwrap()).await;

    let user2 = Keypair::new();
    let user_mining2 = test_rewards_pool
        .initialize_mining(&mut context, &user2.pubkey())
        .await;
    let lockup_period = LockupPeriod::ThreeMonths;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user2.pubkey(),
            &user_mining2,
            100,
            lockup_period,
            &test_rewards_pool.token_mint_pubkey,
        )
        .await
        .unwrap();

    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 1_000_000)
        .await
        .unwrap();

    let user_reward1 = Keypair::new();
    create_token_account(
        &mut context,
        &user_reward1,
        &test_rewards_pool.token_mint_pubkey,
        &user1.pubkey(),
        0,
    )
    .await
    .unwrap();

    test_rewards_pool
        .claim(&mut context, &user1, &user_mining1, &user_reward1.pubkey())
        .await
        .unwrap();

    let user_reward2 = Keypair::new();
    create_token_account(
        &mut context,
        &user_reward2,
        &test_rewards_pool.token_mint_pubkey,
        &user2.pubkey(),
        0,
    )
    .await
    .unwrap();

    test_rewards_pool
        .claim(&mut context, &user2, &user_mining2, &user_reward2.pubkey())
        .await
        .unwrap();

    let user_reward_account1 = get_account(&mut context, &user_reward1.pubkey()).await;
    let user_reward1 = Account::unpack(user_reward_account1.data.borrow()).unwrap();

    assert_eq!(user_reward1.amount, 333_333);

    let user_reward_account2 = get_account(&mut context, &user_reward2.pubkey()).await;
    let user_reward2 = Account::unpack(user_reward_account2.data.borrow()).unwrap();

    assert_eq!(user_reward2.amount, 666_666);
}

#[tokio::test]
// User 1: lockup for ThreeMonth, 5 distributions, 1 claim
// User 2: lockup for OneYear, 5 distributions, 5 claims
async fn multiple_consequantial_distributions_for_two_user() {
    let setup = setup().await;
    let (mut context, test_rewards_pool, user1, user_mining1, rewarder) = setup;
    let user_reward1 = Keypair::new();
    create_token_account(
        &mut context,
        &user_reward1,
        &test_rewards_pool.token_mint_pubkey,
        &user1.pubkey(),
        0,
    )
    .await
    .unwrap();

    let user2 = Keypair::new();
    let user_mining2 = test_rewards_pool
        .initialize_mining(&mut context, &user2.pubkey())
        .await;
    let lockup_period = LockupPeriod::OneYear;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user2.pubkey(),
            &user_mining2,
            100,
            lockup_period,
            &test_rewards_pool.token_mint_pubkey,
        )
        .await
        .unwrap();
    let user_reward2 = Keypair::new();
    create_token_account(
        &mut context,
        &user_reward2,
        &test_rewards_pool.token_mint_pubkey,
        &user2.pubkey(),
        0,
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
            .claim(&mut context, &user2, &user_mining2, &user_reward2.pubkey())
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
        .claim(&mut context, &user2, &user_mining2, &user_reward2.pubkey())
        .await
        .unwrap();

    test_rewards_pool
        .claim(&mut context, &user1, &user_mining1, &user_reward1.pubkey())
        .await
        .unwrap();

    let user_reward_account1 = get_account(&mut context, &user_reward1.pubkey()).await;
    let user_reward1 = Account::unpack(user_reward_account1.data.borrow()).unwrap();
    assert_eq!(user_reward1.amount, 125);

    let user_reward_account2 = get_account(&mut context, &user_reward2.pubkey()).await;
    let user_reward2 = Account::unpack(user_reward_account2.data.borrow()).unwrap();

    assert_eq!(user_reward2.amount, 375);
}
