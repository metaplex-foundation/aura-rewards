// use crate::utils::*;
// use mplx_rewards::utils::LockupPeriod;
// use solana_program::pubkey::Pubkey;
// use solana_program_test::*;
// use solana_sdk::{signature::Keypair, signer::Signer, system_transaction::transfer};

// async fn setup() -> (ProgramTestContext, TestRewards, Keypair, Pubkey) {
//     let test = ProgramTest::new(
//         "mplx_rewards",
//         mplx_rewards::id(),
//         processor!(mplx_rewards::processor::process_instruction),
//     );

//     let mut context = test.start_with_context().await;
//     let deposit_token_mint = Keypair::new();
//     let payer = &context.payer.pubkey();
//     create_mint(&mut context, &deposit_token_mint, payer)
//         .await
//         .unwrap();

//     let test_reward_pool = TestRewards::new(deposit_token_mint.pubkey());

//     test_reward_pool
//         .initialize_pool(&mut context)
//         .await
//         .unwrap();

//     let mining_owner = Keypair::new();
//     let user_mining = test_reward_pool
//         .initialize_mining(&mut context, &mining_owner.pubkey())
//         .await;
//     let tx = transfer(
//         &context.payer,
//         &mining_owner.pubkey(),
//         1000000000, // 1 SOL
//         context.last_blockhash,
//     );
//     context.banks_client.process_transaction(tx).await.unwrap();
//     let tx = transfer(
//         &context.payer,
//         &test_reward_pool.distribution_authority.pubkey(),
//         1000000000, // 1 SOL
//         context.last_blockhash,
//     );
//     context.banks_client.process_transaction(tx).await.unwrap();

//     (context, test_reward_pool, mining_owner, user_mining)
// }

// #[tokio::test]
// async fn success_on_depositing() {
//     // TODO: this test is time-consuming. Feel free to enable it only before commiting.
//     let (mut context, test_rewards, mining_owner, mining) = setup().await;

//     for _ in 0..300 {
//         test_rewards
//             .deposit_mining(
//                 &mut context,
//                 &mining,
//                 1,
//                 LockupPeriod::ThreeMonths,
//                 &mining_owner,
//             )
//             .await
//             .unwrap();
//         advance_clock_by_ts(&mut context, 86_400).await;
//     }
// }

// #[tokio::test]
// async fn success_on_distributing() {
//     // TODO: this test is time-consuming. Feel free to enable it only before commiting.
//     let (mut context, test_rewards, mining_owner, mining) = setup().await;

//     // mint token for fill_authority aka wallet who will fill the vault with tokens
//     let rewarder = Keypair::new();
//     create_token_account(
//         &mut context,
//         &rewarder,
//         &test_rewards.token_mint_pubkey,
//         &test_rewards.fill_authority.pubkey(),
//         0,
//     )
//     .await
//     .unwrap();
//     mint_tokens(
//         &mut context,
//         &test_rewards.token_mint_pubkey,
//         &rewarder.pubkey(),
//         1000000,
//     )
//     .await
//     .unwrap();

//     test_rewards
//         .deposit_mining(
//             &mut context,
//             &mining,
//             1,
//             LockupPeriod::ThreeMonths,
//             &mining_owner,
//         )
//         .await
//         .unwrap();
//     test_rewards
//         .fill_vault(&mut context, &rewarder.pubkey(), 1000000, u64::MAX)
//         .await
//         .unwrap();

//     for _ in 0..300 {
//         test_rewards.distribute_rewards(&mut context).await.unwrap();
//         advance_clock_by_ts(&mut context, 86_400).await;
//     }
// }
