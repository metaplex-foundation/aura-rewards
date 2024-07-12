use std::borrow::Borrow;

use crate::utils::*;
use mplx_rewards::{error::MplxRewardsError, state::Mining, utils::LockupPeriod};
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError, program_pack::Pack, signature::Keypair, signer::Signer,
    transaction::TransactionError,
};

async fn setup() -> (ProgramTestContext, TestRewards, Keypair, Pubkey) {
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

    let mining_owner = Keypair::new();
    let user_mining = test_reward_pool
        .initialize_mining(&mut context, &mining_owner.pubkey())
        .await;

    (context, test_reward_pool, mining_owner, user_mining)
}

#[tokio::test]
async fn success() {
    let (mut context, test_rewards, mining_owner, mining) = setup().await;
    let mining_owner_before = context
        .banks_client
        .get_account(mining_owner.pubkey())
        .await
        .unwrap();
    assert_eq!(None, mining_owner_before);

    test_rewards
        .deposit_mining(
            &mut context,
            &mining,
            100,
            LockupPeriod::ThreeMonths,
            &mining_owner.pubkey(),
            &mining,
        )
        .await
        .unwrap();

    test_rewards
        .close_mining(&mut context, &mining, &mining_owner, &mining_owner.pubkey())
        .await
        .unwrap();

    let mining_account_after = context.banks_client.get_account(mining).await.unwrap();
    assert_eq!(None, mining_account_after);

    let mining_owner = get_account(&mut context, &mining_owner.pubkey()).await;
    assert!(mining_owner.lamports > 0);
}

#[tokio::test]
async fn forbing_closing_if_stake_from_others_is_not_zero() {
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

    let mining_owner_before = context
        .banks_client
        .get_account(mining_owner.pubkey())
        .await
        .unwrap();
    assert_eq!(None, mining_owner_before);

    test_rewards
        .deposit_mining(
            &mut context,
            &mining,
            100,
            LockupPeriod::ThreeMonths,
            &mining_owner.pubkey(),
            &delegate_mining,
        )
        .await
        .unwrap();

    let res = test_rewards
        .close_mining(
            &mut context,
            &delegate_mining,
            &delegate,
            &delegate.pubkey(),
        )
        .await;

    match res {
        Err(BanksClientError::TransactionError(TransactionError::InstructionError(
            _,
            InstructionError::Custom(code),
        ))) => {
            assert_eq!(code, MplxRewardsError::StakeFromOthersMustBeZero as u32);
        }
        _ => unreachable!(),
    }
}
