use crate::utils::{assert_custom_on_chain_error::AssertCustomOnChainErr, *};
use mplx_rewards::{error::MplxRewardsError, utils::LockupPeriod};
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};
use spl_token::state::Account;
use std::borrow::Borrow;

async fn setup() -> (ProgramTestContext, TestRewards) {
    let test = ProgramTest::default();

    let mut context = test.start_with_context().await;
    let owner = &context.payer.pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, owner).await.unwrap();

    let test_reward_pool = TestRewards::new(mint.pubkey());
    test_reward_pool
        .initialize_pool(&mut context)
        .await
        .unwrap();

    let user = Keypair::new();
    let user_mining = test_reward_pool
        .initialize_mining(&mut context, &user.pubkey())
        .await;
    test_reward_pool
        .deposit_mining(
            &mut context,
            &user_mining,
            100,
            LockupPeriod::ThreeMonths,
            &user.pubkey(),
            &user_mining,
        )
        .await
        .unwrap();

    let account = Keypair::new();
    create_token_account(
        &mut context,
        &account,
        &test_reward_pool.token_mint_pubkey,
        &user.pubkey(),
        0,
    )
    .await
    .unwrap();

    (context, test_reward_pool)
}

#[tokio::test]
async fn success() {
    let (mut context, test_rewards) = setup().await;
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

    // calculate distribution_ens time
    let distribution_ends_at = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64
        + 86400 * 100;

    test_rewards
        .fill_vault(&mut context, &rewarder.pubkey(), 100, distribution_ends_at)
        .await
        .unwrap();

    let vault_account = get_account(&mut context, &test_rewards.vault_pubkey).await;
    let rewarder_account = get_account(&mut context, &rewarder.pubkey()).await;

    let vault = Account::unpack(vault_account.data.borrow()).unwrap();
    let rewarder = Account::unpack(rewarder_account.data.borrow()).unwrap();

    assert_eq!(vault.amount, 100);
    assert_eq!(rewarder.amount, 0);
}

#[tokio::test]
async fn zero_amount_of_rewards() {
    let (mut context, test_rewards) = setup().await;

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

    let distribution_ends_at = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64
        + 86400 * 100;

    test_rewards
        .fill_vault(&mut context, &rewarder.pubkey(), 0, distribution_ends_at)
        .await
        .assert_on_chain_err(MplxRewardsError::RewardsMustBeGreaterThanZero);
}
