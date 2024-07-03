use crate::utils::*;
use mplx_rewards::state::Mining;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

async fn setup() -> (ProgramTestContext, TestRewards) {
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

    (context, test_reward_pool)
}

#[tokio::test]
async fn success() {
    let (mut context, test_rewards) = setup().await;

    let user = Keypair::new();
    let user_mining = test_rewards
        .initialize_mining(&mut context, &user.pubkey())
        .await;

    let mining_account = get_account(&mut context, &user_mining).await;
    let mining = deserialize_account::<Mining>(mining_account);

    assert_eq!(mining.reward_pool, test_rewards.reward_pool);
    assert_eq!(mining.owner, user.pubkey());
}
