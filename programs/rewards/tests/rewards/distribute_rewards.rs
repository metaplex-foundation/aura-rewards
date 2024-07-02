use crate::utils::*;
use mplx_rewards::utils::LockupPeriod;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{clock::SECONDS_PER_DAY, signature::Keypair, signer::Signer};

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
        100,
    )
    .await
    .unwrap();

    (context, test_rewards, rewarder.pubkey())
}

#[tokio::test]
async fn happy_path() {
    let (mut context, test_rewards, rewarder) = setup().await;

    let (user, user_rewards, user_mining_addr) = create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_addr,
            100,
            LockupPeriod::ThreeMonths,
            &user.pubkey(),
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
        + SECONDS_PER_DAY * 100;

    test_rewards
        .fill_vault(&mut context, &rewarder, 100, distribution_ends_at)
        .await
        .unwrap();
    // distribute rewards to users
    test_rewards.distribute_rewards(&mut context).await.unwrap();

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

    assert_tokens(&mut context, &user_rewards.pubkey(), 1).await;
}

#[tokio::test]
async fn happy_path_with_flex() {
    let (mut context, test_rewards, rewarder) = setup().await;

    let (user, user_rewards, user_mining_addr) = create_end_user(&mut context, &test_rewards).await;
    test_rewards
        .deposit_mining(
            &mut context,
            &user_mining_addr,
            100,
            LockupPeriod::Flex,
            &user.pubkey(),
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
        + SECONDS_PER_DAY * 100;

    test_rewards
        .fill_vault(&mut context, &rewarder, 100, distribution_ends_at)
        .await
        .unwrap();
    // distribute rewards to users
    test_rewards.distribute_rewards(&mut context).await.unwrap();

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

    assert_tokens(&mut context, &user_rewards.pubkey(), 1).await;
}