use crate::utils::*;
use mplx_rewards::{
    state::{Mining, RewardPool},
    utils::LockupPeriod,
};
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
    let owner = &context.payer.pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, owner).await.unwrap();

    let test_reward_pool = TestRewards::new(mint.pubkey());
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

    let lockup_period = LockupPeriod::ThreeMonths;
    test_rewards
        .deposit_mining(
            &mut context,
            &mining,
            100,
            lockup_period,
            &mining,
            &mining_owner,
        )
        .await
        .unwrap();

    test_rewards
        .withdraw_mining(&mut context, &mining, &mining, 30, &mining_owner.pubkey())
        .await
        .unwrap();

    let reward_pool =
        deserialize_account::<RewardPool>(&mut context, &test_rewards.reward_pool).await;
    assert_eq!(reward_pool.total_share, 170);

    let mining = deserialize_account::<Mining>(&mut context, &mining).await;
    assert_eq!(mining.share, 170);
}

#[tokio::test]
async fn success_with_5kkk_after_expiring() {
    let (mut context, test_rewards, mining_owner, mining) = setup().await;

    let lockup_period = LockupPeriod::ThreeMonths;
    test_rewards
        .deposit_mining(
            &mut context,
            &mining,
            5000000000,
            lockup_period,
            &mining,
            &mining_owner,
        )
        .await
        .unwrap();

    advance_clock_by_ts(&mut context, (100 * SECONDS_PER_DAY).try_into().unwrap()).await;

    test_rewards
        .withdraw_mining(
            &mut context,
            &mining,
            &mining,
            5000000000,
            &mining_owner.pubkey(),
        )
        .await
        .unwrap();

    let reward_pool =
        deserialize_account::<RewardPool>(&mut context, &test_rewards.reward_pool).await;
    assert_eq!(reward_pool.total_share, 0);

    let mining = deserialize_account::<Mining>(&mut context, &mining).await;
    assert_eq!(mining.share, 0);
}
