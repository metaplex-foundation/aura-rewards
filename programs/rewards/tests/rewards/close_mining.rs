use std::borrow::BorrowMut;

use crate::utils::*;
use assert_custom_on_chain_error::AssertCustomOnChainErr;
use mplx_rewards::{error::MplxRewardsError, state::WrappedMining, utils::LockupPeriod};
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{clock::SECONDS_PER_DAY, signature::Keypair, signer::Signer};

async fn setup() -> (ProgramTestContext, TestRewards, Keypair, Pubkey) {
    let test = ProgramTest::new("mplx_rewards", mplx_rewards::ID, None);
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
        .initialize_mining(&mut context, &mining_owner)
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
            &mining,
            &mining_owner.pubkey()
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
async fn close_when_has_stake_from_others() {
    let (mut context, test_rewards, mining_owner, mining) = setup().await;

    let delegate = Keypair::new();
    let delegate_mining = test_rewards
        .initialize_mining(&mut context, &delegate)
        .await;
    test_rewards
        .deposit_mining(
            &mut context,
            &delegate_mining,
            3_000_000, // 18_000_000 of weighted stake
            LockupPeriod::OneYear,
            &delegate.pubkey(),
            &delegate_mining,
            &delegate.pubkey()
        )
        .await
        .unwrap();
    let mut delegate_mining_account = get_account(&mut context, &delegate_mining).await;
    let d_mining_data = &mut delegate_mining_account.data.borrow_mut();
    let d_wrapped_mining = WrappedMining::from_bytes_mut(d_mining_data).unwrap();
    assert_eq!(d_wrapped_mining.mining.share, 18_000_000);
    assert_eq!(d_wrapped_mining.mining.stake_from_others, 0);

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
            &delegate_mining,
            &delegate.pubkey()
        )
        .await
        .unwrap();

    test_rewards
        .close_mining(
            &mut context,
            &delegate_mining,
            &delegate,
            &delegate.pubkey(),
        )
        .await
        .assert_on_chain_err(MplxRewardsError::StakeFromOthersMustBeZero);
}

#[tokio::test]
async fn close_when_has_unclaimed_rewards() {
    let (mut context, test_rewards, mining_owner, mining) = setup().await;

    test_rewards
        .deposit_mining(
            &mut context,
            &mining,
            100,
            LockupPeriod::ThreeMonths,
            &mining_owner.pubkey(),
            &mining,
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
