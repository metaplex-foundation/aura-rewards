use crate::utils::*;
use assert_custom_on_chain_error::AssertCustomOnChainErr;
use mplx_rewards::utils::LockupPeriod;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

async fn setup() -> (ProgramTestContext, TestRewards, Pubkey) {
    let test = ProgramTest::new(
        "mplx_rewards",
        mplx_rewards::id(),
        processor!(mplx_rewards::processor::process_instruction),
    );
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
async fn change_delegate_to_the_same() {
    let (mut context, test_rewards, _) = setup().await;

    let (user_a, _, user_mining_a) = create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_a,
            6_000_000,
            LockupPeriod::OneYear,
            &user_a.pubkey(),
            &user_mining_a,
        )
        .await
        .unwrap();
    test_rewards
        .change_delegate(
            &mut context,
            &user_mining_a,
            &user_a,
            &user_mining_a,
            &user_mining_a,
            6_000_000,
        )
        .await
        .assert_on_chain_err(mplx_rewards::error::MplxRewardsError::DelegatesAreTheSame);
}
