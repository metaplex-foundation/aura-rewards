use crate::utils::*;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use spl_token::state::Account;
use std::borrow::Borrow;

async fn setup() -> (
    ProgramTestContext,
    TestRewards,
    Keypair,
    Pubkey,
    Pubkey,
    Pubkey,
) {
    let (mut context, _) = presetup().await;

    let owner = &context.payer.pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, owner).await.unwrap();

    let test_reward_pool = TestRewards::new(Some(mint.pubkey()));
    test_reward_pool
        .initialize_pool(&mut context)
        .await
        .unwrap();

    let user = Keypair::new();
    let user_mining = test_reward_pool
        .initialize_mining(&mut context, &user.pubkey())
        .await;
    test_reward_pool
        .deposit_mining(&mut context, &user.pubkey(), &user_mining, 100)
        .await
        .unwrap();

    let rewarder = Keypair::new();
    create_token_account(&mut context, &rewarder, &mint.pubkey(), owner, 0)
        .await
        .unwrap();
    mint_tokens(&mut context, &mint.pubkey(), &rewarder.pubkey(), 1_000_000)
        .await
        .unwrap();

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

    test_reward_pool
        .add_vault(&mut context, &fee_keypair.pubkey())
        .await;

    (
        context,
        test_reward_pool,
        user,
        user_mining,
        fee_keypair.pubkey(),
        rewarder.pubkey(),
    )
}

#[tokio::test]
async fn success() {
    let (mut context, test_rewards, user, user_mining, fee, rewarder) = setup().await;

    test_rewards
        .fill_vault(&mut context, &fee, &rewarder, 1_000_000)
        .await
        .unwrap();

    let user_reward = Keypair::new();
    create_token_account(
        &mut context,
        &user_reward,
        &test_rewards.token_mint_pubkey,
        &user.pubkey(),
        0,
    )
    .await
    .unwrap();

    test_rewards
        .claim(&mut context, &user, &user_mining, &user_reward.pubkey())
        .await
        .unwrap();

    let user_reward_account = get_account(&mut context, &user_reward.pubkey()).await;
    let user_reward = Account::unpack(user_reward_account.data.borrow()).unwrap();

    assert_eq!(user_reward.amount, 980_000);
}

#[tokio::test]
async fn with_two_users() {
    let (mut context, test_rewards, user1, user_mining1, fee, rewarder) = setup().await;

    let user2 = Keypair::new();
    let user_mining2 = test_rewards
        .initialize_mining(&mut context, &user2.pubkey())
        .await;
    test_rewards
        .deposit_mining(&mut context, &user2.pubkey(), &user_mining2, 50)
        .await
        .unwrap();

    test_rewards
        .fill_vault(&mut context, &fee, &rewarder, 1_000_000)
        .await
        .unwrap();

    let user_reward1 = Keypair::new();
    create_token_account(
        &mut context,
        &user_reward1,
        &test_rewards.token_mint_pubkey,
        &user1.pubkey(),
        0,
    )
    .await
    .unwrap();

    test_rewards
        .claim(&mut context, &user1, &user_mining1, &user_reward1.pubkey())
        .await
        .unwrap();

    let user_reward2 = Keypair::new();
    create_token_account(
        &mut context,
        &user_reward2,
        &test_rewards.token_mint_pubkey,
        &user2.pubkey(),
        0,
    )
    .await
    .unwrap();

    test_rewards
        .claim(&mut context, &user2, &user_mining2, &user_reward2.pubkey())
        .await
        .unwrap();

    let user_reward_account1 = get_account(&mut context, &user_reward1.pubkey()).await;
    let user_reward1 = Account::unpack(user_reward_account1.data.borrow()).unwrap();

    assert_eq!(user_reward1.amount, 653_333);

    let user_reward_account2 = get_account(&mut context, &user_reward2.pubkey()).await;
    let user_reward2 = Account::unpack(user_reward_account2.data.borrow()).unwrap();

    assert_eq!(user_reward2.amount, 326_666);
}
