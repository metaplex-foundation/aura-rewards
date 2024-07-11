use crate::utils::*;
use mplx_rewards::state::RewardPool;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

async fn setup() -> (ProgramTestContext, TestRewards) {
    let test = ProgramTest::new(
        "mplx_rewards",
        mplx_rewards::id(),
        processor!(mplx_rewards::processor::process_instruction),
    );

    let mut context = test.start_with_context().await;

    let mint_owner = &context.payer.pubkey();
    let reward_mint = Keypair::new();
    create_mint(&mut context, &reward_mint, mint_owner)
        .await
        .unwrap();

    let test_rewards = TestRewards::new(reward_mint.pubkey());

    (context, test_rewards)
}

#[tokio::test]
async fn success() {
    let (mut context, test_rewards) = setup().await;

    test_rewards.initialize_pool(&mut context).await.unwrap();

    let reward_pool_account = get_account(&mut context, &test_rewards.reward_pool).await;
    let reward_pool = deserialize_account::<RewardPool>(reward_pool_account);

    assert_eq!(
        reward_pool.deposit_authority,
        test_rewards.deposit_authority.pubkey()
    );
    assert_eq!(
        reward_pool.fill_authority,
        test_rewards.fill_authority.pubkey()
    );
    assert_eq!(
        reward_pool.calculator.reward_mint,
        test_rewards.token_mint_pubkey
    );
}
