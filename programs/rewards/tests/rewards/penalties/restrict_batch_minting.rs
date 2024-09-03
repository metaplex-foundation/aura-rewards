use crate::utils::*;
use mplx_rewards::state::WrappedImmutableMining;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};
use std::borrow::Borrow;

async fn setup() -> (ProgramTestContext, TestRewards, Pubkey) {
    let test = ProgramTest::new("mplx_rewards", mplx_rewards::ID, None);
    let mut context = test.start_with_context().await;

    let owner = &context.payer.pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, owner).await.unwrap();

    let test_rewards = TestRewards::new(mint.pubkey());
    test_rewards.initialize_pool(&mut context).await.unwrap();

    // mint token for fill_authority aka wallet who will fill the vault with tokens
    let rewarder = Keypair::new();
    create_token_account(
        &mut context,
        &rewarder,
        &test_rewards.token_mint_pubkey,
        &test_rewards.fill_authority.pubkey(),
        0,
    )
    .await
    .unwrap();
    mint_tokens(
        &mut context,
        &test_rewards.token_mint_pubkey,
        &rewarder.pubkey(),
        1_000_000,
    )
    .await
    .unwrap();

    (context, test_rewards, rewarder.pubkey())
}

#[tokio::test]
async fn batch_minting_restricted() {
    let (mut context, test_rewards, _) = setup().await;

    let (user_a, _, user_mining_a) = create_end_user(&mut context, &test_rewards).await;

    test_rewards
        .restrict_batch_minting(&mut context, &user_mining_a, 100, &user_a.pubkey())
        .await
        .unwrap();

    let mining_account = get_account(&mut context, &user_mining_a).await;
    let mining_data = &mining_account.data.borrow();
    let mining = WrappedImmutableMining::from_bytes(mining_data).unwrap();
    assert_eq!(mining.mining.batch_minting_restricted_until, 100);
}
