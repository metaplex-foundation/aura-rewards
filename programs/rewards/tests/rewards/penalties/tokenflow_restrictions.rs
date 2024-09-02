use crate::utils::*;
use assert_custom_on_chain_error::AssertCustomOnChainErr;
use mplx_rewards::{
    state::{WrappedMining, WrappedRewardPool},
    utils::LockupPeriod,
};
use solana_program::{program_pack::Pack, pubkey::Pubkey};
use solana_program_test::*;
use solana_sdk::{clock::SECONDS_PER_DAY, signature::Keypair, signer::Signer};
use spl_token::state::Account;
use std::borrow::{Borrow, BorrowMut};

async fn setup() -> (ProgramTestContext, TestRewards, Pubkey) {
    let test = ProgramTest::new("mplx_rewards", mplx_rewards::ID, None);
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
async fn claim_restricted() {
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
            &user_a.pubkey(),
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

    // restrict claiming
    test_rewards
        .restrict_tokenflow(&mut context, &user_mining_a, &user_a.pubkey())
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
        .assert_on_chain_err(mplx_rewards::error::MplxRewardsError::ClaimingRestricted);
}

#[tokio::test]
async fn claim_allowed() {
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
            &user_a.pubkey(),
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

    // restrict claiming
    test_rewards
        .restrict_tokenflow(&mut context, &user_mining_a, &user_a.pubkey())
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
        .assert_on_chain_err(mplx_rewards::error::MplxRewardsError::ClaimingRestricted);

    // allow claiming
    test_rewards
        .allow_tokenflow(&mut context, &user_mining_a, &user_a.pubkey())
        .await
        .unwrap();

    // MUST TO ADVANCE TO AVOID CACHING
    advance_clock_by_ts(&mut context, 2).await;

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

    assert_eq!(user_rewards_a.amount, 100);
}

#[tokio::test]
async fn withdraw_restricted() {
    let (mut context, test_rewards, rewarder) = setup().await;

    let (user_a, _, user_mining_a) = create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_a,
            100,
            LockupPeriod::ThreeMonths,
            &user_a.pubkey(),
            &user_mining_a,
            &user_a.pubkey(),
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

    // restrict claiming
    test_rewards
        .restrict_tokenflow(&mut context, &user_mining_a, &user_a.pubkey())
        .await
        .unwrap();

    test_rewards
        .withdraw_mining(
            &mut context,
            &user_mining_a,
            &user_mining_a,
            100,
            &user_a.pubkey(),
            &user_a.pubkey(),
        )
        .await
        .assert_on_chain_err(mplx_rewards::error::MplxRewardsError::WithdrawalRestricted);
}

#[tokio::test]
async fn withdraw_allowed() {
    let (mut context, test_rewards, rewarder) = setup().await;

    let (user_a, _, user_mining_a) = create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_a,
            100,
            LockupPeriod::ThreeMonths,
            &user_a.pubkey(),
            &user_mining_a,
            &user_a.pubkey(),
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

    // restrict claiming
    test_rewards
        .restrict_tokenflow(&mut context, &user_mining_a, &user_a.pubkey())
        .await
        .unwrap();

    test_rewards
        .withdraw_mining(
            &mut context,
            &user_mining_a,
            &user_mining_a,
            100,
            &user_a.pubkey(),
            &user_a.pubkey(),
        )
        .await
        .assert_on_chain_err(mplx_rewards::error::MplxRewardsError::WithdrawalRestricted);

    // prevent caching
    advance_clock_by_ts(&mut context, (distribution_ends_at + 1).try_into().unwrap()).await;
    test_rewards
        .allow_tokenflow(&mut context, &user_mining_a, &user_a.pubkey())
        .await
        .unwrap();

    test_rewards
        .withdraw_mining(
            &mut context,
            &user_mining_a,
            &user_mining_a,
            100,
            &user_a.pubkey(),
            &user_a.pubkey(),
        )
        .await
        .unwrap();

    let mut reward_pool_account =
        get_account(&mut context, &test_rewards.reward_pool.pubkey()).await;
    let reward_pool_data = &mut reward_pool_account.data.borrow_mut();
    let wrapped_reward_pool = WrappedRewardPool::from_bytes_mut(reward_pool_data).unwrap();
    let reward_pool = wrapped_reward_pool.pool;

    assert_eq!(reward_pool.total_share, 0);

    let mut mining_account = get_account(&mut context, &user_mining_a).await;
    let mining_data = &mut mining_account.data.borrow_mut();
    let wrapped_mining = WrappedMining::from_bytes_mut(mining_data).unwrap();
    assert_eq!(wrapped_mining.mining.share, 0);
}
