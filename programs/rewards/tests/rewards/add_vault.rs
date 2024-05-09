use std::borrow::Borrow;

use mplx_rewards::state::RewardPool;
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

use crate::utils::{create_mint, create_token_account, get_account, TestRewards};

async fn setup() -> (ProgramTestContext, TestRewards, Keypair) {
    let (mut context, token_mint) = async {
        let test = ProgramTest::new(
            "mplx_rewards",
            mplx_rewards::id(),
            processor!(mplx_rewards::processor::process_instruction),
        );

        let mut context = test.start_with_context().await;
        let payer_pubkey = context.payer.pubkey();

        // // TODO: check liquidity ming
        let liquidity_mint = Keypair::new();
        create_mint(&mut context, &liquidity_mint, &payer_pubkey)
            .await
            .unwrap();
        (context, liquidity_mint)
    }
    .await;

    let test_reward_pool = TestRewards::new(Some(token_mint.pubkey()));

    test_reward_pool
        .initialize_pool(&mut context)
        .await
        .unwrap();

    let user = Keypair::new();
    let fee_keypair = Keypair::new();

    create_token_account(
        &mut context,
        &fee_keypair,
        &test_reward_pool.token_mint_pubkey,
        &user.pubkey(),
        0,
    )
    .await
    .unwrap();

    (context, test_reward_pool, fee_keypair)
}

#[tokio::test]
async fn success() {
    let (mut context, test_rewards, fee_keypair) = setup().await;
    test_rewards
        .add_vault(&mut context, &fee_keypair.pubkey())
        .await;

    let reward_pool_account = get_account(&mut context, &test_rewards.mining_reward_pool).await;
    let reward_pool = RewardPool::unpack(reward_pool_account.data.borrow()).unwrap();
    let vaults = reward_pool.vaults.first().unwrap();

    assert_eq!(vaults.fee_account, fee_keypair.pubkey());
    assert_eq!(vaults.reward_mint, test_rewards.token_mint_pubkey);
}
