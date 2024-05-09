use crate::utils::*;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use spl_token::state::Account;
use std::borrow::Borrow;

async fn setup() -> (ProgramTestContext, TestRewards, Pubkey, Pubkey, Pubkey) {
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

    let vault = test_reward_pool
        .add_vault(&mut context, &fee_keypair.pubkey())
        .await;

    (
        context,
        test_reward_pool,
        vault,
        fee_keypair.pubkey(),
        rewarder.pubkey(),
    )
}

#[tokio::test]
async fn success() {
    let (mut context, test_rewards, vault, fee, rewarder) = setup().await;

    test_rewards
        .fill_vault(&mut context, &fee, &rewarder, 1_000_000)
        .await
        .unwrap();

    let vault_account = get_account(&mut context, &vault).await;
    let fee_account = get_account(&mut context, &fee).await;
    let rewarder_account = get_account(&mut context, &rewarder).await;

    let vault = Account::unpack(vault_account.data.borrow()).unwrap();
    let fee = Account::unpack(fee_account.data.borrow()).unwrap();
    let rewarder = Account::unpack(rewarder_account.data.borrow()).unwrap();

    assert_eq!(vault.amount, 980_000);
    assert_eq!(fee.amount, 20_000);
    assert_eq!(rewarder.amount, 0);
}
