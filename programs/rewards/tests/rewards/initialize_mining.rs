use crate::utils::*;
use mplx_rewards::state::WrappedMining;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};
use std::borrow::BorrowMut;

async fn setup() -> (ProgramTestContext, TestRewards) {
    let test = ProgramTest::new("mplx_rewards", mplx_rewards::ID, None);
    let mut context = test.start_with_context().await;

    let owner = &context.payer.pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, owner).await.unwrap();

    let test_reward_pool = TestRewards::new(mint.pubkey());
    test_reward_pool
        .initialize_pool(&mut context)
        .await
        .unwrap();

    (context, test_reward_pool)
}

#[tokio::test]
async fn success() {
    let (mut context, test_rewards) = setup().await;

    let user = Keypair::new();
    let user_mining = test_rewards
        .initialize_mining(&mut context, &user.pubkey())
        .await;

    let mut mining_account = get_account(&mut context, &user_mining).await;
    let mining_data = &mut mining_account.data.borrow_mut();
    let wrapped_mining = WrappedMining::from_bytes_mut(mining_data).unwrap();

    assert_eq!(wrapped_mining.mining.reward_pool, test_rewards.reward_pool);
    assert_eq!(wrapped_mining.mining.owner, user.pubkey());
}
