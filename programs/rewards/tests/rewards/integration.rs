use crate::utils::*;
use mplx_rewards::utils::LockupPeriod;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::clock::SECONDS_PER_DAY;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

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
            LockupPeriod::SixMonths,
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
    advance_clock_by_ts(&mut context, (5 * SECONDS_PER_DAY).try_into().unwrap()).await;
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
    claim_and_assert(
        &test_rewards_pool,
        &mut context,
        &user_a,
        &user_mining_a,
        &user_reward_a.pubkey(),
        386,
    )
    .await;
    claim_and_assert(
        &test_rewards_pool,
        &mut context,
        &user_b,
        &user_mining_b,
        &user_reward_b.pubkey(),
        681,
    )
    .await;
    claim_and_assert(
        &test_rewards_pool,
        &mut context,
        &user_c,
        &user_mining_c,
        &user_reward_c.pubkey(),
        131,
    )
    .await;

    // D3 expires (therefore turns into Flex)
    advance_clock_by_ts(&mut context, (171 * SECONDS_PER_DAY).try_into().unwrap()).await;

    // User B unstakes and claims D3
    test_rewards_pool
        .withdraw_mining(&mut context, &user_b.pubkey(), &user_mining_b, 100_000)
        .await
        .unwrap();

    // 1 distribution happens
    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();

    claim_and_assert(
        &test_rewards_pool,
        &mut context,
        &user_a,
        &user_mining_a,
        &user_reward_a.pubkey(),
        390,
    )
    .await;
    claim_and_assert(
        &test_rewards_pool,
        &mut context,
        &user_b,
        &user_mining_b,
        &user_reward_b.pubkey(),
        776,
    )
    .await;
    claim_and_assert(
        &test_rewards_pool,
        &mut context,
        &user_c,
        &user_mining_c,
        &user_reward_c.pubkey(),
        131,
    )
    .await;

    // 1 distribution happens
    advance_clock_by_ts(&mut context, SECONDS_PER_DAY.try_into().unwrap()).await;
    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();
    claim_and_assert(
        &test_rewards_pool,
        &mut context,
        &user_a,
        &user_mining_a,
        &user_reward_a.pubkey(),
        394,
    )
    .await;
    claim_and_assert(
        &test_rewards_pool,
        &mut context,
        &user_b,
        &user_mining_b,
        &user_reward_b.pubkey(),
        871,
    )
    .await;
    claim_and_assert(
        &test_rewards_pool,
        &mut context,
        &user_c,
        &user_mining_c,
        &user_reward_c.pubkey(),
        131,
    )
    .await;

    // D4 expires.
    advance_clock_by_ts(&mut context, (5 * SECONDS_PER_DAY).try_into().unwrap()).await;
    // 2 distributions happen
    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();
    claim_and_assert(
        &test_rewards_pool,
        &mut context,
        &user_a,
        &user_mining_a,
        &user_reward_a.pubkey(),
        409,
    )
    .await;
    claim_and_assert(
        &test_rewards_pool,
        &mut context,
        &user_c,
        &user_mining_c,
        &user_reward_c.pubkey(),
        131,
    )
    .await;

    advance_clock_by_ts(&mut context, SECONDS_PER_DAY.try_into().unwrap()).await;
    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();

    // D0 expires
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
    claim_and_assert(
        &test_rewards_pool,
        &mut context,
        &user_a,
        &user_mining_a,
        &user_reward_a.pubkey(),
        441,
    )
    .await;
    // Usr B unstakes and claims D4
    test_rewards_pool
        .withdraw_mining(&mut context, &user_b.pubkey(), &user_mining_b, 100_000)
        .await
        .unwrap();
    claim_and_assert(
        &test_rewards_pool,
        &mut context,
        &user_b,
        &user_mining_b,
        &user_reward_b.pubkey(),
        1621,
    )
    .await;
    // 1 distribution happens
    test_rewards_pool
        .fill_vault(&mut context, &rewarder, 100)
        .await
        .unwrap();
    // User C claims his rewards
    claim_and_assert(
        &test_rewards_pool,
        &mut context,
        &user_c,
        &user_mining_c,
        &user_reward_c.pubkey(),
        231,
    )
    .await;
}
