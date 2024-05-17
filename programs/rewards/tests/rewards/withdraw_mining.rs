use crate::utils::*;
use mplx_rewards::state::{Mining, RewardPool};
use mplx_rewards::utils::LockupPeriod;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::borrow::Borrow;

async fn setup() -> (ProgramTestContext, TestRewards, Pubkey, Pubkey, Pubkey) {
    let (mut context, _) = presetup().await;
    let owner = &context.payer.pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, owner).await.unwrap();

    let test_reward_pool = TestRewards::new(Some(mint.pubkey()));
    test_reward_pool
        .initialize_pool(&mut context)
        .await
        .unwrap();

    let user = Keypair::new();
    let user_mining = test_reward_pool
        .initialize_mining(&mut context, &user.pubkey())
        .await;
    test_reward_pool.add_vault(&mut context).await;

    (
        context,
        test_reward_pool,
        user.pubkey(),
        user_mining,
        mint.pubkey(),
    )
}

#[tokio::test]
async fn success() {
    let (mut context, test_rewards, user, mining, mint) = setup().await;

    let lockup_period = LockupPeriod::ThreeMonths;
    test_rewards
        .deposit_mining(&mut context, &user, &mining, 100, lockup_period, &mint)
        .await
        .unwrap();

    test_rewards
        .withdraw_mining(&mut context, &user, &mining, 30)
        .await
        .unwrap();

    let reward_pool_account = get_account(&mut context, &test_rewards.mining_reward_pool).await;
    let reward_pool = RewardPool::unpack(reward_pool_account.data.borrow()).unwrap();

    assert_eq!(reward_pool.total_share, 170);

    let mining_account = get_account(&mut context, &mining).await;
    let mining = Mining::unpack(mining_account.data.borrow()).unwrap();
    assert_eq!(mining.share, 170);
}
