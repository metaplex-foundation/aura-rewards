use std::borrow::{Borrow, BorrowMut};

use borsh::BorshDeserialize;
use mplx_rewards::utils::LockupPeriod;
use solana_program::pubkey::Pubkey;
use solana_program_test::{BanksClientError, ProgramTestContext};
use solana_sdk::{
    account::Account,
    program_pack::Pack,
    signature::{Keypair, Signer},
    system_instruction::{self},
    transaction::{Transaction, TransactionError},
};
use spl_token::state::Account as SplTokenAccount;

pub type BanksClientResult<T> = Result<T, BanksClientError>;

#[derive(Debug)]
pub struct TestRewards {
    pub token_mint_pubkey: Pubkey,
    pub deposit_authority: Keypair,
    pub distribution_authority: Keypair,
    pub fill_authority: Keypair,
    pub reward_pool: Pubkey,
    pub vault_pubkey: Pubkey,
}

impl TestRewards {
    pub fn new(token_mint_pubkey: Pubkey) -> Self {
        let deposit_authority = Keypair::new();
        let fill_authority = Keypair::new();
        let distribution_authority = Keypair::new();

        let (reward_pool, _) = Pubkey::find_program_address(
            &[
                b"reward_pool".as_ref(),
                &deposit_authority.pubkey().to_bytes(),
            ],
            &mplx_rewards::id(),
        );

        let (vault_pubkey, _vault_bump) = Pubkey::find_program_address(
            &[
                b"vault".as_ref(),
                &reward_pool.to_bytes(),
                &token_mint_pubkey.to_bytes(),
            ],
            &mplx_rewards::id(),
        );

        Self {
            token_mint_pubkey,
            deposit_authority,
            fill_authority,
            reward_pool,
            vault_pubkey,
            distribution_authority,
        }
    }

    pub async fn initialize_pool(&self, context: &mut ProgramTestContext) -> BanksClientResult<()> {
        // Initialize mining pool
        let tx = Transaction::new_signed_with_payer(
            &[mplx_rewards::instruction::initialize_pool(
                &mplx_rewards::id(),
                &self.reward_pool,
                &self.token_mint_pubkey,
                &self.vault_pubkey,
                &context.payer.pubkey(),
                &self.deposit_authority.pubkey(),
                &self.fill_authority.pubkey(),
                &self.distribution_authority.pubkey(),
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.deposit_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn initialize_mining(
        &self,
        context: &mut ProgramTestContext,
        mining_owner: &Pubkey,
    ) -> Pubkey {
        let (mining_account, _) = Pubkey::find_program_address(
            &[
                b"mining".as_ref(),
                mining_owner.as_ref(),
                self.reward_pool.as_ref(),
            ],
            &mplx_rewards::id(),
        );

        let tx = Transaction::new_signed_with_payer(
            &[mplx_rewards::instruction::initialize_mining(
                &mplx_rewards::id(),
                &self.reward_pool,
                &mining_account,
                &context.payer.pubkey(),
                mining_owner,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        mining_account
    }

    pub async fn change_delegate(
        &self,
        context: &mut ProgramTestContext,
        mining: &Pubkey,
        mining_owner: &Keypair,
        new_delegate_mining: &Pubkey,
        old_delegate_mining: &Pubkey,
        amount: u64,
    ) -> BanksClientResult<()> {
        let tx = Transaction::new_signed_with_payer(
            &[mplx_rewards::instruction::change_delegate(
                &mplx_rewards::id(),
                &self.reward_pool,
                mining,
                &self.deposit_authority.pubkey(),
                &mining_owner.pubkey(),
                old_delegate_mining,
                new_delegate_mining,
                amount,
            )],
            Some(&context.payer.pubkey()),
            &[&self.deposit_authority, mining_owner, &context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn deposit_mining(
        &self,
        context: &mut ProgramTestContext,
        mining_account: &Pubkey,
        amount: u64,
        lockup_period: LockupPeriod,
        owner: &Pubkey,
        delegate_mining: &Pubkey,
        mining_owner: &Keypair,
    ) -> BanksClientResult<()> {
        let tx = Transaction::new_signed_with_payer(
            &[mplx_rewards::instruction::deposit_mining(
                &mplx_rewards::id(),
                &self.reward_pool,
                mining_account,
                &self.deposit_authority.pubkey(),
                delegate_mining,
                &mining_owner.pubkey(),
                amount,
                lockup_period,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.deposit_authority, mining_owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn withdraw_mining(
        &self,
        context: &mut ProgramTestContext,
        mining_account: &Pubkey,
        delegate_mining: &Pubkey,
        amount: u64,
        owner: &Pubkey,
    ) -> BanksClientResult<()> {
        let tx = Transaction::new_signed_with_payer(
            &[mplx_rewards::instruction::withdraw_mining(
                &mplx_rewards::id(),
                &self.reward_pool,
                mining_account,
                &self.deposit_authority.pubkey(),
                delegate_mining,
                amount,
                owner,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.deposit_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn fill_vault(
        &self,
        context: &mut ProgramTestContext,
        from: &Pubkey,
        amount: u64,
        distribution_ends_at: u64,
    ) -> BanksClientResult<()> {
        let tx = Transaction::new_signed_with_payer(
            &[mplx_rewards::instruction::fill_vault(
                &mplx_rewards::id(),
                &self.reward_pool,
                &self.token_mint_pubkey,
                &self.vault_pubkey,
                &self.fill_authority.pubkey(),
                from,
                amount,
                distribution_ends_at,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.fill_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn claim(
        &self,
        context: &mut ProgramTestContext,
        user: &Keypair,
        mining_account: &Pubkey,
        user_reward_token: &Pubkey,
    ) -> BanksClientResult<()> {
        let tx = Transaction::new_signed_with_payer(
            &[mplx_rewards::instruction::claim(
                &mplx_rewards::id(),
                &self.reward_pool,
                &self.token_mint_pubkey,
                &self.vault_pubkey,
                mining_account,
                &user.pubkey(),
                &self.deposit_authority.pubkey(),
                user_reward_token,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, user, &self.deposit_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn distribute_rewards(
        &self,
        context: &mut ProgramTestContext,
    ) -> BanksClientResult<()> {
        let tx = Transaction::new_signed_with_payer(
            &[mplx_rewards::instruction::distribute_rewards(
                &mplx_rewards::id(),
                &self.reward_pool,
                &self.distribution_authority.pubkey(),
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.distribution_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn extend_stake(
        &self,
        context: &mut ProgramTestContext,
        mining_account: &Pubkey,
        delegate_mining: &Pubkey,
        mining_owner: &Keypair,
        old_lockup_period: LockupPeriod,
        new_lockup_period: LockupPeriod,
        deposit_start_ts: u64,
        base_amount: u64,
        additional_amount: u64,
    ) -> BanksClientResult<()> {
        let tx = Transaction::new_signed_with_payer(
            &[mplx_rewards::instruction::extend_stake(
                &mplx_rewards::id(),
                &self.reward_pool,
                mining_account,
                &self.deposit_authority.pubkey(),
                delegate_mining,
                &mining_owner.pubkey(),
                old_lockup_period,
                new_lockup_period,
                deposit_start_ts,
                base_amount,
                additional_amount,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.deposit_authority, mining_owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn close_mining(
        &self,
        context: &mut ProgramTestContext,
        mining_account: &Pubkey,
        mining_owner: &Keypair,
        target_account: &Pubkey,
    ) -> BanksClientResult<()> {
        let tx = Transaction::new_signed_with_payer(
            &[mplx_rewards::instruction::close_mining(
                &mplx_rewards::id(),
                mining_account,
                &mining_owner.pubkey(),
                target_account,
                &self.deposit_authority.pubkey(),
                &self.reward_pool,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.deposit_authority, mining_owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}

pub async fn create_token_account(
    context: &mut ProgramTestContext,
    account: &Keypair,
    mint: &Pubkey,
    manager: &Pubkey,
    lamports: u64,
) -> BanksClientResult<()> {
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &account.pubkey(),
                rent.minimum_balance(spl_token::state::Account::LEN) + lamports,
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &account.pubkey(),
                mint,
                manager,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, account],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn get_account(context: &mut ProgramTestContext, pubkey: &Pubkey) -> Account {
    context
        .banks_client
        .get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty")
}

pub async fn create_mint(
    context: &mut ProgramTestContext,
    mint: &Keypair,
    manager: &Pubkey,
) -> BanksClientResult<()> {
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                manager,
                None,
                0,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, mint],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn mint_tokens(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    account: &Pubkey,
    amount: u64,
) -> BanksClientResult<()> {
    let tx = Transaction::new_signed_with_payer(
        &[spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            account,
            &context.payer.pubkey(),
            &[],
            amount,
        )
        .unwrap()],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn advance_clock_by_ts(context: &mut ProgramTestContext, ts: i64) -> i64 {
    let old_clock = context
        .banks_client
        .get_sysvar::<solana_program::clock::Clock>()
        .await
        .unwrap();

    let initial_slot = context.banks_client.get_root_slot().await.unwrap();
    context
        .warp_to_slot(initial_slot + (ts / 2) as u64)
        .unwrap();

    let mut new_clock = old_clock.clone();
    new_clock.unix_timestamp += ts;
    context.borrow_mut().set_sysvar(&new_clock);
    new_clock.unix_timestamp
}

pub async fn create_end_user(
    context: &mut ProgramTestContext,
    test_rewards: &TestRewards,
) -> (Keypair, Keypair, Pubkey) {
    let user = Keypair::new();
    let user_reward = Keypair::new();
    create_token_account(
        context,
        &user_reward,
        &test_rewards.token_mint_pubkey,
        &user.pubkey(),
        0,
    )
    .await
    .unwrap();
    let user_mining = test_rewards
        .initialize_mining(context, &user.pubkey())
        .await;

    (user, user_reward, user_mining)
}

pub async fn assert_tokens(context: &mut ProgramTestContext, reward_account: &Pubkey, amount: u64) {
    let user_reward_account: Account = get_account(context, reward_account).await;
    let user_reward = SplTokenAccount::unpack(user_reward_account.data.borrow()).unwrap();
    assert_eq!(user_reward.amount, amount);
}

pub async fn claim_and_assert(
    test_rewards_pool: &TestRewards,
    context: &mut ProgramTestContext,
    user: &Keypair,
    user_mining: &Pubkey,
    user_reward: &Pubkey,
    amount: u64,
) {
    test_rewards_pool
        .claim(context, user, user_mining, user_reward)
        .await
        .unwrap();
    assert_tokens(context, user_reward, amount).await;
}

pub mod assert_custom_on_chain_error {
    use super::*;
    use std::fmt::Debug;

    pub trait AssertCustomOnChainErr {
        fn assert_on_chain_err(self, expected_err: MplxRewardsError);
    }

    impl<T: Debug> AssertCustomOnChainErr for Result<T, BanksClientError> {
        fn assert_on_chain_err(self, expected_err: MplxRewardsError) {
            assert!(self.is_err());
            match self.unwrap_err() {
                BanksClientError::TransactionError(TransactionError::InstructionError(
                    _,
                    InstructionError::Custom(code),
                )) => {
                    debug_assert_eq!(expected_err as u32, code);
                }
                _ => unreachable!("BanksClientError has no 'Custom' variant."),
            }
        }
    }
}

pub fn deserialize_account<T: BorshDeserialize>(account: Account) -> T {
    let mut bytes: &[u8] = (*account.data).borrow();
    T::deserialize(&mut bytes).unwrap()
}
