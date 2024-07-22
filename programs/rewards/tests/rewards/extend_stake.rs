use crate::utils::*;
use mplx_rewards::{
    state::{Mining, RewardPool},
    utils::LockupPeriod,
};
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{clock::SECONDS_PER_DAY, program_pack::Pack, signature::Keypair, signer::Signer};
use std::borrow::Borrow;

async fn setup() -> (ProgramTestContext, TestRewards, Pubkey, Pubkey) {
    let test = ProgramTest::new(
        "mplx_rewards",
        mplx_rewards::id(),
        processor!(mplx_rewards::processor::process_instruction),
    );

    let mut context = test.start_with_context().await;
    let deposit_token_mint = Keypair::new();
    let payer = &context.payer.pubkey();
    create_mint(&mut context, &deposit_token_mint, payer)
        .await
        .unwrap();

    let test_reward_pool = TestRewards::new(deposit_token_mint.pubkey());

    test_reward_pool
        .initialize_pool(&mut context)
        .await
        .unwrap();

    let user = Keypair::new();
    let user_mining = test_reward_pool
        .initialize_mining(&mut context, &user.pubkey())
        .await;

    (context, test_reward_pool, user.pubkey(), user_mining)
}

#[tokio::test]
async fn restake_before_its_expired() {
    let (mut context, test_rewards, mining_owner, mining) = setup().await;

    let deposit_start_ts = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64;
    let base_amount = 100;
    let additional_amount = 100;
    let old_lockup_period = LockupPeriod::ThreeMonths;
    let new_lockup_period = LockupPeriod::ThreeMonths;

    test_rewards
        .deposit_mining(
            &mut context,
            &mining,
            100,
            old_lockup_period,
            &mining_owner,
            &mining,
        )
        .await
        .unwrap();

    // advance for ten days
    let curr_ts =
        advance_clock_by_ts(&mut context, (10 * SECONDS_PER_DAY).try_into().unwrap()).await;

    test_rewards
        .extend_stake(
            &mut context,
            &mining,
            &mining,
            old_lockup_period,
            new_lockup_period,
            deposit_start_ts,
            base_amount,
            additional_amount,
            &mining_owner,
        )
        .await
        .unwrap();

    // new expiration date modifier added
    let beginning_of_the_old_expiration_day = LockupPeriod::ThreeMonths
        .end_timestamp(deposit_start_ts - (deposit_start_ts % SECONDS_PER_DAY))
        .unwrap();
    check_modifier_at_a_day(&mut context, mining, 0, beginning_of_the_old_expiration_day).await;

    // new expiration date modifier added
    let beginning_of_the_expiration_day = LockupPeriod::ThreeMonths
        .end_timestamp(curr_ts as u64)
        .unwrap();
    check_modifier_at_a_day(&mut context, mining, 200, beginning_of_the_expiration_day).await;

    // and power is multiplied twice
    check_weighted_stake(&mut context, mining, 400).await;
}

#[tokio::test]
async fn restake_for_another_period_after_old_is_expired() {
    let (mut context, test_rewards, mining_owner, mining) = setup().await;

    let deposit_start_ts = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64;
    let base_amount = 100;
    let additional_amount = 100;

    test_rewards
        .deposit_mining(
            &mut context,
            &mining,
            100,
            LockupPeriod::ThreeMonths,
            &mining_owner,
            &mining,
        )
        .await
        .unwrap();

    let curr_ts =
        advance_clock_by_ts(&mut context, (91 * SECONDS_PER_DAY).try_into().unwrap()).await;

    // we set it to Flex and not to ThreeMonth because it's expired
    let old_lockup_period = LockupPeriod::Flex;
    let new_lockup_period = LockupPeriod::OneYear;

    test_rewards
        .extend_stake(
            &mut context,
            &mining,
            &mining,
            old_lockup_period,
            new_lockup_period,
            deposit_start_ts,
            base_amount,
            additional_amount,
            &mining_owner,
        )
        .await
        .unwrap();

    // new expiration date modifier added
    let beginning_of_the_expiration_day =
        LockupPeriod::OneYear.end_timestamp(curr_ts as u64).unwrap();
    check_modifier_at_a_day(&mut context, mining, 1000, beginning_of_the_expiration_day).await;

    // and power is multiplied twice
    check_weighted_stake(&mut context, mining, 1200).await;
}

#[tokio::test]
async fn just_prolong_without_adding_tokes() {
    let (mut context, test_rewards, mining_owner, mining) = setup().await;

    let deposit_start_ts = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64;
    let base_amount = 100;
    let additional_amount = 0;
    let old_lockup_period = LockupPeriod::ThreeMonths;
    let new_lockup_period = LockupPeriod::ThreeMonths;

    test_rewards
        .deposit_mining(
            &mut context,
            &mining,
            100,
            old_lockup_period,
            &mining_owner,
            &mining,
        )
        .await
        .unwrap();

    // advance for ten days
    let curr_ts =
        advance_clock_by_ts(&mut context, (10 * SECONDS_PER_DAY).try_into().unwrap()).await;

    test_rewards
        .extend_stake(
            &mut context,
            &mining,
            &mining,
            old_lockup_period,
            new_lockup_period,
            deposit_start_ts,
            base_amount,
            additional_amount,
            &mining_owner,
        )
        .await
        .unwrap();

    // new expiration date modifier added
    let beginning_of_the_old_expiration_day = LockupPeriod::ThreeMonths
        .end_timestamp(deposit_start_ts - (deposit_start_ts % SECONDS_PER_DAY))
        .unwrap();
    check_modifier_at_a_day(&mut context, mining, 0, beginning_of_the_old_expiration_day).await;

    // new expiration date modifier added
    let beginning_of_the_expiration_day = LockupPeriod::ThreeMonths
        .end_timestamp(curr_ts as u64)
        .unwrap();
    check_modifier_at_a_day(&mut context, mining, 100, beginning_of_the_expiration_day).await;

    // and power is multiplied twice
    check_weighted_stake(&mut context, mining, 200).await;
}

#[tokio::test]
async fn restake_after_its_expired_with_no_additional_tokens() {
    let (mut context, test_rewards, mining_owner, mining) = setup().await;

    let deposit_start_ts = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64;
    let base_amount = 100;
    let additional_amount = 0;

    test_rewards
        .deposit_mining(
            &mut context,
            &mining,
            100,
            LockupPeriod::ThreeMonths,
            &mining_owner,
            &mining,
        )
        .await
        .unwrap();

    let curr_ts =
        advance_clock_by_ts(&mut context, (91 * SECONDS_PER_DAY).try_into().unwrap()).await;

    // we set it to Flex and not to ThreeMonth because it's expired
    let old_lockup_period = LockupPeriod::Flex;
    let new_lockup_period = LockupPeriod::ThreeMonths;

    test_rewards
        .extend_stake(
            &mut context,
            &mining,
            &mining,
            old_lockup_period,
            new_lockup_period,
            deposit_start_ts,
            base_amount,
            additional_amount,
            &mining_owner,
        )
        .await
        .unwrap();

    // new expiration date modifier added
    let beginning_of_the_expiration_day = LockupPeriod::ThreeMonths
        .end_timestamp(curr_ts as u64)
        .unwrap();
    check_modifier_at_a_day(&mut context, mining, 100, beginning_of_the_expiration_day).await;

    // and power is multiplied twice
    check_weighted_stake(&mut context, mining, 200).await;
}

#[tokio::test]
async fn restake_in_expiration_day() {
    let (mut context, test_rewards, mining_owner, mining) = setup().await;

    let deposit_start_ts = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64;
    let base_amount = 100;
    let additional_amount = 0;

    test_rewards
        .deposit_mining(
            &mut context,
            &mining,
            100,
            LockupPeriod::ThreeMonths,
            &mining_owner,
            &mining,
        )
        .await
        .unwrap();

    let curr_ts =
        advance_clock_by_ts(&mut context, (90 * SECONDS_PER_DAY).try_into().unwrap()).await;

    // we set it to Flex and not to ThreeMonth because it's expired
    let old_lockup_period = LockupPeriod::Flex;
    let new_lockup_period = LockupPeriod::ThreeMonths;

    test_rewards
        .extend_stake(
            &mut context,
            &mining,
            &mining,
            old_lockup_period,
            new_lockup_period,
            deposit_start_ts,
            base_amount,
            additional_amount,
            &mining_owner,
        )
        .await
        .unwrap();

    // new expiration date modifier added
    let beginning_of_the_expiration_day = LockupPeriod::ThreeMonths
        .end_timestamp(curr_ts as u64)
        .unwrap();
    check_modifier_at_a_day(&mut context, mining, 100, beginning_of_the_expiration_day).await;

    // and power is multiplied twice
    check_weighted_stake(&mut context, mining, 200).await;
}

#[tokio::test]
async fn prolong_with_delegate() {
    let (mut context, test_rewards, mining_owner, mining) = setup().await;

    let delegate = Keypair::new();
    let delegate_mining = test_rewards
        .initialize_mining(&mut context, &delegate.pubkey())
        .await;
    test_rewards
        .deposit_mining(
            &mut context,
            &delegate_mining,
            3_000_000, // 18_000_000 of weighted stake
            LockupPeriod::OneYear,
            &delegate.pubkey(),
            &delegate_mining,
        )
        .await
        .unwrap();
    let delegate_mining_account = get_account(&mut context, &delegate_mining).await;
    let d_mining = Mining::unpack(delegate_mining_account.data.borrow()).unwrap();
    assert_eq!(d_mining.share, 18_000_000);
    assert_eq!(d_mining.stake_from_others, 0);

    let deposit_start_ts = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64;
    let base_amount = 100;
    let additional_amount = 0;
    let old_lockup_period = LockupPeriod::ThreeMonths;
    let new_lockup_period = LockupPeriod::ThreeMonths;

    test_rewards
        .deposit_mining(
            &mut context,
            &mining,
            base_amount,
            old_lockup_period,
            &mining_owner,
            &delegate_mining,
        )
        .await
        .unwrap();
    let mining_account = get_account(&mut context, &mining).await;
    let mining_unpacked = Mining::unpack(mining_account.data.borrow()).unwrap();
    assert_eq!(mining_unpacked.share, 200);
    assert_eq!(mining_unpacked.stake_from_others, 0);

    let delegate_mining_account = get_account(&mut context, &delegate_mining).await;
    let d_mining = Mining::unpack(delegate_mining_account.data.borrow()).unwrap();
    assert_eq!(d_mining.share, 18_000_000);
    assert_eq!(d_mining.stake_from_others, 100);

    let reward_pool_acc = get_account(&mut context, &test_rewards.reward_pool).await;
    let reward_pool_unpacked = RewardPool::unpack(reward_pool_acc.data.borrow()).unwrap();
    assert_eq!(reward_pool_unpacked.total_share, 18_000_300);

    // advance for ten days
    let curr_ts =
        advance_clock_by_ts(&mut context, (10 * SECONDS_PER_DAY).try_into().unwrap()).await;

    test_rewards
        .extend_stake(
            &mut context,
            &mining,
            &delegate_mining,
            old_lockup_period,
            new_lockup_period,
            deposit_start_ts,
            base_amount,
            additional_amount,
            &mining_owner,
        )
        .await
        .unwrap();

    // new expiration date modifier added
    let beginning_of_the_old_expiration_day = LockupPeriod::ThreeMonths
        .end_timestamp(deposit_start_ts - (deposit_start_ts % SECONDS_PER_DAY))
        .unwrap();
    check_modifier_at_a_day(&mut context, mining, 0, beginning_of_the_old_expiration_day).await;

    // new expiration date modifier added
    let beginning_of_the_expiration_day = LockupPeriod::ThreeMonths
        .end_timestamp(curr_ts as u64)
        .unwrap();
    check_modifier_at_a_day(&mut context, mining, 100, beginning_of_the_expiration_day).await;

    // and power is multiplied twice
    check_weighted_stake(&mut context, mining, 200).await;
}

pub async fn check_weighted_stake(
    context: &mut ProgramTestContext,
    mining_account: Pubkey,
    expected_share: u64,
) {
    let mining_account = get_account(context, &mining_account).await;
    let mining = Mining::unpack(mining_account.data.borrow()).unwrap();
    assert_eq!(mining.share, expected_share);
}

pub async fn check_modifier_at_a_day(
    context: &mut ProgramTestContext,
    mining_account: Pubkey,
    expected_modifier: u64,
    day_to_check: u64,
) {
    let mining_account = get_account(context, &mining_account).await;
    let mining = Mining::unpack(mining_account.data.borrow()).unwrap();

    let expiration_modifier_for_day = mining
        .index
        .weighted_stake_diffs
        .get(&day_to_check)
        .unwrap_or_else(|| panic!("Modifier for date: {:?} must exist", day_to_check));

    assert_eq!(*expiration_modifier_for_day, expected_modifier);
}
