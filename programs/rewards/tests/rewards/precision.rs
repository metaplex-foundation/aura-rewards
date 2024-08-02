use crate::utils::*;
use mplx_rewards::utils::LockupPeriod;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{clock::SECONDS_PER_DAY, signature::Keypair, signer::Signer};

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
        5000 * 1_000_000,
    )
    .await
    .unwrap();

    (context, test_rewards, rewarder.pubkey())
}

#[tokio::test]
async fn precision_distribution() {
    let (mut context, test_rewards, rewarder) = setup().await;

    let (rich_user, _rich_user_rewards, rich_user_mining_addr) =
        create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &rich_user_mining_addr,
            333_333_316 * 1_000_000,
            LockupPeriod::Flex,
            &rich_user.pubkey(),
            &rich_user_mining_addr,
        )
        .await
        .unwrap();

    let (user, user_rewards, user_mining_addr) = create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_addr,
            17 * 1_000_000,
            LockupPeriod::Flex,
            &user.pubkey(),
            &user_mining_addr,
        )
        .await
        .unwrap();

    // fill vault with tokens
    let distribution_ends_at = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64
        + SECONDS_PER_DAY * 30;

    test_rewards
        .fill_vault(
            &mut context,
            &rewarder,
            5000 * 1_000_000,
            distribution_ends_at,
        )
        .await
        .unwrap();
    // distribute rewards to users

    for _ in 0..30 {
        test_rewards.distribute_rewards(&mut context).await.unwrap();
        advance_clock_by_ts(&mut context, SECONDS_PER_DAY.try_into().unwrap()).await;
    }

    // // user claims their rewards
    test_rewards
        .claim(
            &mut context,
            &user,
            &user_mining_addr,
            &user_rewards.pubkey(),
        )
        .await
        .unwrap();

    assert_tokens(&mut context, &user_rewards.pubkey(), 254).await;
}
