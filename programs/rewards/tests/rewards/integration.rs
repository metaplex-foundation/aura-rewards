use std::borrow::Borrow;

use crate::utils::*;
use mplx_rewards::utils::LockupPeriod;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::clock::SECONDS_PER_DAY;
use solana_sdk::program_pack::Pack;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use spl_token::state::Account;

async fn setup() -> (ProgramTestContext, TestRewards, Pubkey, Keypair) {
    let (mut context, _) = presetup().await;

    let owner = &context.payer.pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, owner).await.unwrap();

    let test_reward_pool = TestRewards::new(Some(mint.pubkey()));
    test_reward_pool
        .initialize_pool(&mut context)
        .await
        .unwrap();

    let rewarder = Keypair::new();
    create_token_account(&mut context, &rewarder, &mint.pubkey(), owner, 0)
        .await
        .unwrap();
    mint_tokens(&mut context, &mint.pubkey(), &rewarder.pubkey(), 1_000_000)
        .await
        .unwrap();

    test_reward_pool.add_vault(&mut context).await;

    (context, test_reward_pool, rewarder.pubkey(), mint)
}

#[tokio::test]
async fn success() {
    let (mut context, test_rewards_pool, rewarder, mint) = setup().await;

    let (user_a, user_reward_a, user_mining_a) =
        create_user(&mut context, &test_rewards_pool).await;
    let (user_b, user_reward_b, user_mining_b) =
        create_user(&mut context, &test_rewards_pool).await;
    let (user_c, user_reward_c, user_mining_c) =
        create_user(&mut context, &test_rewards_pool).await;

    // User C - deposit (D0) 100 tokens for 1 year
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_c.pubkey(),
            &user_mining_c,
            100,
            LockupPeriod::OneYear,
            &mint.pubkey(),
        )
        .await
        .unwrap();
    // 1 distribuiton happens
    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();
    // User A - deposit(D1) 1000 tokens for 1 year.
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_a.pubkey(),
            &user_mining_a,
            1000,
            LockupPeriod::OneYear,
            &mint.pubkey(),
        )
        .await
        .unwrap();
    // 3 distributions happen
    for _ in 0..3 {
        advance_clock_by_ts(&mut context, SECONDS_PER_DAY.try_into().unwrap()).await;
        test_rewards_pool
            .fill_vault(&mut context, &rewarder, 100)
            .await
            .unwrap();
    }
    // User A - deposit(D2) 2000 tokens for 1 year.
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_a.pubkey(),
            &user_mining_a,
            2000,
            LockupPeriod::OneYear,
            &mint.pubkey(),
        )
        .await
        .unwrap();
    // 1 distribution happens
    advance_clock_by_ts(&mut context, SECONDS_PER_DAY.try_into().unwrap()).await;
    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();
    // User B deposit(D3) 100k tokens 3 month after the User A for half a year
    advance_clock_by_ts(&mut context, (SECONDS_PER_DAY * 90).try_into().unwrap()).await;
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_b.pubkey(),
            &user_mining_b,
            100_000,
            LockupPeriod::ThreeMonths,
            &mint.pubkey(),
        )
        .await
        .unwrap();
    // 1 distribution happens
    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();
    // User B deposit(D4) 100k tokens for half a year - his unclaimed balance is calculated
    test_rewards_pool
        .deposit_mining(
            &mut context,
            &user_b.pubkey(),
            &user_mining_b,
            100_000,
            LockupPeriod::SixMonths,
            &mint.pubkey(),
        )
        .await
        .unwrap();
    // 6 distributions happen
    for _ in 0..6 {
        advance_clock_by_ts(&mut context, SECONDS_PER_DAY.try_into().unwrap()).await;
        test_rewards_pool
            .fill_vault(&mut context, &rewarder, 100)
            .await
            .unwrap();
    }

    // D3 expires
    advance_clock_by_ts(&mut context, (84 * SECONDS_PER_DAY).try_into().unwrap()).await;
    // 1 distribution happens
    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();
    // User B unstakes and claims D3
    // TODO: test user B claimed amount
    test_rewards_pool
        .withdraw_mining(&mut context, &user_b.pubkey(), &user_mining_b, 100_000)
        .await
        .unwrap();
    test_rewards_pool
        .claim(
            &mut context,
            &user_b,
            &user_mining_b,
            &user_reward_b.pubkey(),
        )
        .await
        .unwrap();
    assert_tokens(&mut context, &user_reward_b.pubkey(), 855).await;

    // 1 distribution happens
    advance_clock_by_ts(&mut context, SECONDS_PER_DAY.try_into().unwrap()).await;
    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();
    // D4 expires (180 - 90 - 1 = 89).
    advance_clock_by_ts(&mut context, (89 * SECONDS_PER_DAY).try_into().unwrap()).await;
    // 2 distributions happen
    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();
    advance_clock_by_ts(&mut context, SECONDS_PER_DAY.try_into().unwrap()).await;

    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();
    // D0 expires (365 - 275 = 90)
    // D1 expires
    advance_clock_by_ts(&mut context, (90 * SECONDS_PER_DAY).try_into().unwrap()).await;
    // 1 distribution happens
    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();
    // D2 expires
    advance_clock_by_ts(&mut context, (3 * SECONDS_PER_DAY).try_into().unwrap()).await;
    // 5 distributions happen
    for _ in 0..5 {
        test_rewards_pool
            .fill_vault(&mut context, &rewarder, 100)
            .await
            .unwrap();
        advance_clock_by_ts(&mut context, SECONDS_PER_DAY.try_into().unwrap()).await;
    }

    // User A unstakes and claims D1 and D2
    test_rewards_pool
        .withdraw_mining(&mut context, &user_a.pubkey(), &user_mining_a, 3000)
        .await
        .unwrap();
    test_rewards_pool
        .claim(
            &mut context,
            &user_a,
            &user_mining_a,
            &user_reward_a.pubkey(),
        )
        .await
        .unwrap();
    assert_tokens(&mut context, &user_reward_a.pubkey(), 947).await;
    // Usr B unstakes and claims D4
    test_rewards_pool
        .withdraw_mining(&mut context, &user_b.pubkey(), &user_mining_b, 100_000)
        .await
        .unwrap();
    test_rewards_pool
        .claim(
            &mut context,
            &user_b,
            &user_mining_b,
            &user_reward_b.pubkey(),
        )
        .await
        .unwrap();
    assert_tokens(&mut context, &user_reward_b.pubkey(), 1071).await;
    // 1 distribution happens
    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();
    // User C claims his rewards
    test_rewards_pool
        .claim(
            &mut context,
            &user_c,
            &user_mining_c,
            &user_reward_c.pubkey(),
        )
        .await
        .unwrap();
    assert_tokens(&mut context, &user_reward_c.pubkey(), 122).await;
}

async fn create_user(
    context: &mut ProgramTestContext,
    test_rewards_pool: &TestRewards,
) -> (Keypair, Keypair, Pubkey) {
    let user = Keypair::new();
    let user_reward = Keypair::new();
    create_token_account(
        context,
        &user_reward,
        &test_rewards_pool.token_mint_pubkey,
        &user.pubkey(),
        0,
    )
    .await
    .unwrap();
    let user_mining = test_rewards_pool
        .initialize_mining(context, &user.pubkey())
        .await;

    (user, user_reward, user_mining)
}

async fn assert_tokens(context: &mut ProgramTestContext, reward_account: &Pubkey, amount: u64) {
    let user_reward_account_b = get_account(context, reward_account).await;
    let user_reward2 = Account::unpack(user_reward_account_b.data.borrow()).unwrap();
    assert_eq!(user_reward2.amount, amount);
}
