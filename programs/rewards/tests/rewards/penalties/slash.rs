use crate::utils::*;
use mplx_rewards::{
    state::{WrappedMining, WrappedRewardPool},
    utils::LockupPeriod,
};
use sokoban::NodeAllocatorMap;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{clock::SECONDS_PER_DAY, signature::Keypair, signer::Signer};
use std::borrow::BorrowMut;

async fn setup() -> (ProgramTestContext, TestRewards, Pubkey, Pubkey) {
    let test = ProgramTest::new("mplx_rewards", mplx_rewards::ID, None);
    let mut context = test.start_with_context().await;

    let owner = &context.payer.pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, owner).await.unwrap();

    let test_rewards = TestRewards::new(mint.pubkey());
    test_rewards.initialize_pool(&mut context).await.unwrap();

    let user = Keypair::new();
    let user_mining = test_rewards.initialize_mining(&mut context, &user).await;

    (context, test_rewards, user.pubkey(), user_mining)
}

#[tokio::test]
async fn one_stake_for_a_date() {
    let (mut context, test_rewards, user, mining_addr) = setup().await;

    let stake_expiration_date = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64
        + SECONDS_PER_DAY * 30 * 6;
    let stake_expiration_date = stake_expiration_date - stake_expiration_date % SECONDS_PER_DAY;

    let lockup_period = LockupPeriod::SixMonths;
    test_rewards
        .deposit_mining(
            &mut context,
            &mining_addr,
            150,
            lockup_period,
            &user,
            &mining_addr,
            &user,
        )
        .await
        .unwrap();

    // just test both pool and mining states are correct
    let mut reward_pool_account =
        get_account(&mut context, &test_rewards.reward_pool.pubkey()).await;
    let reward_pool_data = &mut reward_pool_account.data.borrow_mut();
    let wrapped_reward_pool = WrappedRewardPool::from_bytes_mut(reward_pool_data).unwrap();
    let reward_pool = wrapped_reward_pool.pool;
    assert_eq!(reward_pool.total_share, 600);
    assert_eq!(
        *wrapped_reward_pool
            .weighted_stake_diffs
            .get(&stake_expiration_date)
            .unwrap(),
        450
    );
    let mut mining_account = get_account(&mut context, &mining_addr).await;
    let mining_data = &mut mining_account.data.borrow_mut();
    let mining = WrappedMining::from_bytes_mut(mining_data).unwrap();
    assert_eq!(mining.mining.share, 600);
    assert_eq!(
        *mining
            .weighted_stake_diffs
            .get(&stake_expiration_date)
            .unwrap(),
        450
    );

    test_rewards
        .slash(
            &mut context,
            &mining_addr,
            &user,
            50,
            200,
            Some(stake_expiration_date),
        )
        .await
        .unwrap();

    let mut reward_pool_account =
        get_account(&mut context, &test_rewards.reward_pool.pubkey()).await;
    let reward_pool_data = &mut reward_pool_account.data.borrow_mut();
    let wrapped_reward_pool = WrappedRewardPool::from_bytes_mut(reward_pool_data).unwrap();
    let reward_pool = wrapped_reward_pool.pool;
    assert_eq!(reward_pool.total_share, 400);
    assert_eq!(
        *wrapped_reward_pool
            .weighted_stake_diffs
            .get(&stake_expiration_date)
            .unwrap(),
        300
    );
    let mut mining_account = get_account(&mut context, &mining_addr).await;
    let mining_data = &mut mining_account.data.borrow_mut();
    let mining = WrappedMining::from_bytes_mut(mining_data).unwrap();
    assert_eq!(mining.mining.share, 400);
    assert_eq!(
        *mining
            .weighted_stake_diffs
            .get(&stake_expiration_date)
            .unwrap(),
        300
    );
}

#[tokio::test]
async fn multiple_stakes_for_a_date_but_one_slashed() {
    let (mut context, test_rewards, user, mining_addr) = setup().await;

    let stake_expiration_date = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64
        + SECONDS_PER_DAY * 365;
    let stake_expiration_date = stake_expiration_date - stake_expiration_date % SECONDS_PER_DAY;

    let lockup_period = LockupPeriod::OneYear;
    test_rewards
        .deposit_mining(
            &mut context,
            &mining_addr,
            200, // 200 x6
            lockup_period,
            &user,
            &mining_addr,
            &user,
        )
        .await
        .unwrap();

    advance_clock_by_ts(&mut context, (SECONDS_PER_DAY * 185).try_into().unwrap()).await;
    let lockup_period = LockupPeriod::SixMonths;
    test_rewards
        .deposit_mining(
            &mut context,
            &mining_addr,
            150, // 150 x4
            lockup_period,
            &user,
            &mining_addr,
            &user,
        )
        .await
        .unwrap();

    let lockup_period = LockupPeriod::Flex;
    test_rewards
        .deposit_mining(
            &mut context,
            &mining_addr,
            100, // 100 x1
            lockup_period,
            &user,
            &mining_addr,
            &user,
        )
        .await
        .unwrap();

    // weighted stake = 150*4 + 200*6 + 100*1 = 1900
    // diff = 1900 - 150 - 200 - 100 = 1450

    // just test both pool and mining states are correct
    let mut reward_pool_account =
        get_account(&mut context, &test_rewards.reward_pool.pubkey()).await;
    let reward_pool_data = &mut reward_pool_account.data.borrow_mut();
    let wrapped_reward_pool = WrappedRewardPool::from_bytes_mut(reward_pool_data).unwrap();
    let reward_pool = wrapped_reward_pool.pool;
    assert_eq!(reward_pool.total_share, 1900);
    assert_eq!(
        *wrapped_reward_pool
            .weighted_stake_diffs
            .get(&stake_expiration_date)
            .unwrap(),
        1450
    );

    let mut mining_account = get_account(&mut context, &mining_addr).await;
    let mining_data = &mut mining_account.data.borrow_mut();
    let mining = WrappedMining::from_bytes_mut(mining_data).unwrap();
    assert_eq!(mining.mining.share, 1900);
    assert_eq!(
        *mining
            .weighted_stake_diffs
            .get(&stake_expiration_date)
            .unwrap(),
        1450
    );

    test_rewards
        .slash(
            &mut context,
            &mining_addr,
            &user,
            50,
            200,
            Some(stake_expiration_date),
        )
        .await
        .unwrap();

    let mut reward_pool_account =
        get_account(&mut context, &test_rewards.reward_pool.pubkey()).await;
    let reward_pool_data = &mut reward_pool_account.data.borrow_mut();
    let wrapped_reward_pool = WrappedRewardPool::from_bytes_mut(reward_pool_data).unwrap();
    let reward_pool = wrapped_reward_pool.pool;
    assert_eq!(reward_pool.total_share, 1700);
    assert_eq!(
        *wrapped_reward_pool
            .weighted_stake_diffs
            .get(&stake_expiration_date)
            .unwrap(),
        1300
    );
    let mut mining_account = get_account(&mut context, &mining_addr).await;
    let mining_data = &mut mining_account.data.borrow_mut();
    let mining = WrappedMining::from_bytes_mut(mining_data).unwrap();
    assert_eq!(mining.mining.share, 1700);
    assert_eq!(
        *mining
            .weighted_stake_diffs
            .get(&stake_expiration_date)
            .unwrap(),
        1300
    );
}
