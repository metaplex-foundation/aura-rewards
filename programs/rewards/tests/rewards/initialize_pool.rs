use crate::utils::*;
use mplx_rewards::state::RewardPool;
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};
use std::borrow::Borrow;

async fn setup() -> (ProgramTestContext, TestRewards) {
    let (mut context, _) = presetup().await;
    let owner = &context.payer.pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, owner).await.unwrap();

    let test_reward_pool = TestRewards::new(Some(mint.pubkey()));

    (context, test_reward_pool)
}

#[tokio::test]
async fn success() {
    let (mut context, test_reward_pool) = setup().await;

    test_reward_pool
        .initialize_pool(&mut context)
        .await
        .unwrap();

    let reward_pool_account = get_account(&mut context, &test_reward_pool.mining_reward_pool).await;
    let reward_pool = RewardPool::unpack(reward_pool_account.data.borrow()).unwrap();

    assert_eq!(
        reward_pool.rewards_root,
        test_reward_pool.rewards_root.pubkey()
    );
    assert_eq!(
        reward_pool.deposit_authority,
        test_reward_pool.deposit_authority.pubkey()
    );
    assert_eq!(
        reward_pool.liquidity_mint,
        test_reward_pool.token_mint_pubkey
    );
}
