use std::borrow::{Borrow, BorrowMut};

use crate::utils::*;
use assert_custom_on_chain_error::AssertCustomOnChainErr;
use mplx_rewards::{
    state::{WrappedMining, WrappedRewardPool},
    utils::LockupPeriod,
};
use solana_program::{program_pack::Pack, pubkey::Pubkey};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};
use spl_token::state::Account;

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
async fn change_delegate_to_the_same() {
    let (mut context, test_rewards, _) = setup().await;

    let (user_a, _, user_mining_a) = create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_a,
            6_000_000,
            LockupPeriod::OneYear,
            &user_a.pubkey(),
            &user_mining_a,
            &user_a.pubkey(),
        )
        .await
        .unwrap();
    test_rewards
        .change_delegate(
            &mut context,
            &user_mining_a,
            &user_a,
            &user_mining_a,
            &user_mining_a,
            &user_a.pubkey(),
            6_000_000,
        )
        .await
        .assert_on_chain_err(mplx_rewards::error::MplxRewardsError::DelegatesAreTheSame);
}

#[tokio::test]
async fn change_delegate_then_claim() {
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
            &delegate.pubkey(),
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
            &user_mining_a,
            &user_a.pubkey(),
        )
        .await
        .unwrap();
    test_rewards
        .change_delegate(
            &mut context,
            &user_mining_a,
            &user_a,
            &delegate_mining,
            &user_mining_a,
            &delegate.pubkey(),
            1_000_000,
        )
        .await
        .unwrap();

    let mut delegate_mining_account = get_account(&mut context, &delegate_mining).await;
    let d_mining_data = &mut delegate_mining_account.data.borrow_mut();
    let d_wrapped_mining = WrappedMining::from_bytes_mut(d_mining_data).unwrap();
    assert_eq!(d_wrapped_mining.mining.share, 18_000_000);
    assert_eq!(d_wrapped_mining.mining.stake_from_others, 1_000_000);

    let mut reward_pool_account =
        get_account(&mut context, &test_rewards.reward_pool.pubkey()).await;
    let reward_pool_data = &mut reward_pool_account.data.borrow_mut();
    let wrapped_reward_pool = WrappedRewardPool::from_bytes_mut(reward_pool_data).unwrap();
    let reward_pool = wrapped_reward_pool.pool;

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
