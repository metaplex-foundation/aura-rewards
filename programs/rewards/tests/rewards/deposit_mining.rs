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
        .deposit_mining(&mut context, &mining, 100, LockupPeriod::ThreeMonths, &user)
        .await
        .unwrap();

    let reward_pool_account = get_account(&mut context, &test_rewards.reward_pool).await;
    let reward_pool = deserialize_account::<RewardPool>(reward_pool_account);

    assert_eq!(reward_pool.total_share, 200);

    let mining_account = get_account(&mut context, &mining).await;
    let mining = deserialize_account::<Mining>(mining_account);
    assert_eq!(mining.share, 200);
}

#[tokio::test]
async fn success_with_flex() {
    let (mut context, test_rewards, user, mining) = setup().await;

    test_rewards
        .deposit_mining(&mut context, &mining, 100, LockupPeriod::Flex, &user)
        .await
        .unwrap();

    let reward_pool_account = get_account(&mut context, &test_rewards.reward_pool).await;
    let reward_pool = deserialize_account::<RewardPool>(reward_pool_account);

    assert_eq!(reward_pool.total_share, 100);

    let mining_account = get_account(&mut context, &mining).await;
    let mining = deserialize_account::<Mining>(mining_account);
    assert_eq!(mining.share, 100);
}
