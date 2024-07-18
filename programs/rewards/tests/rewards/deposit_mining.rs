use crate::utils::*;
use mplx_rewards::{
    state::{Mining, RewardPool},
    utils::LockupPeriod,
};
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

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

    let user = Keypair::new();
    let user_mining = test_reward_pool
        .initialize_mining(&mut context, &user.pubkey())
        .await;

    (context, test_reward_pool, user, user_mining)
}

#[tokio::test]
async fn success() {
    let (mut context, test_rewards, user, mining) = setup().await;

    test_rewards
        .deposit_mining(
            &mut context,
            &mining,
            100,
            LockupPeriod::ThreeMonths,
            &mining,
            &user,
        )
        .await
        .unwrap();

    let reward_pool =
        deserialize_account::<RewardPool>(&mut context, &test_rewards.reward_pool).await;
    assert_eq!(reward_pool.total_share, 200);

    let mining = deserialize_account::<Mining>(&mut context, &mining).await;
    assert_eq!(mining.share, 200);
}

#[tokio::test]
async fn success_with_flex() {
    let (mut context, test_rewards, user, mining) = setup().await;

    test_rewards
        .deposit_mining(
            &mut context,
            &mining,
            100,
            LockupPeriod::Flex,
            &mining,
            &user,
        )
        .await
        .unwrap();

    let reward_pool =
        deserialize_account::<RewardPool>(&mut context, &test_rewards.reward_pool).await;
    assert_eq!(reward_pool.total_share, 100);

    let mining = deserialize_account::<Mining>(&mut context, &mining).await;
    assert_eq!(mining.share, 100);
}

#[tokio::test]
async fn delegating_success() {
    let (mut context, test_rewards, user, mining) = setup().await;

    let delegate = Keypair::new();
    let delegate_mining = test_rewards
        .initialize_mining(&mut context, &delegate.pubkey())
        .await;
    test_rewards
        .deposit_mining(
            &mut context,
            &delegate_mining,
            3_000_000, // 18_000_000 of weighted stake
            LockupPeriod::OneYear,
            &delegate_mining,
            &delegate,
        )
        .await
        .unwrap();
    let d_mining = deserialize_account::<Mining>(&mut context, &delegate_mining).await;
    assert_eq!(d_mining.share, 18_000_000);
    assert_eq!(d_mining.stake_from_others, 0);

    test_rewards
        .deposit_mining(
            &mut context,
            &mining,
            100,
            LockupPeriod::Flex,
            &delegate_mining,
            &user,
        )
        .await
        .unwrap();

    let d_mining = deserialize_account::<Mining>(&mut context, &delegate_mining).await;
    assert_eq!(d_mining.share, 18_000_000);
    assert_eq!(d_mining.stake_from_others, 100);

    let reward_pool =
        deserialize_account::<RewardPool>(&mut context, &test_rewards.reward_pool).await;

    assert_eq!(reward_pool.total_share, 18_000_200);

    let mining = deserialize_account::<Mining>(&mut context, &mining).await;
    assert_eq!(mining.share, 100);
}
