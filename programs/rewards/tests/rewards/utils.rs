use solana_program::pubkey::Pubkey;
use solana_program_test::{processor, BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::account::Account;
use solana_sdk::program_pack::Pack;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use solana_sdk::transport::TransportError;

use crate::instructions::*;

pub type BanksClientResult<T> = Result<T, TransportError>;

pub async fn transfer(
    context: &mut ProgramTestContext,
    pubkey: &Pubkey,
    amount: u64,
) -> BanksClientResult<()> {
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &context.payer.pubkey(),
            pubkey,
            amount,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

#[derive(Debug)]
pub struct TestRewards {
    pub token_mint_pubkey: Pubkey,
    pub rewards_root: Keypair,
    pub deposit_authority: Keypair,
    pub root_authority: Keypair,
    pub mining_reward_pool: Pubkey,
}

impl TestRewards {
    pub fn new(token_mint_pubkey: Option<Pubkey>) -> Self {
        let token_mint_pubkey = token_mint_pubkey.unwrap();

        let deposit_authority = Keypair::new();
        let rewards_root = Keypair::new();
        let root_authority = Keypair::new();

        let (mining_reward_pool, _) = Pubkey::find_program_address(
            &[
                b"reward_pool".as_ref(),
                &rewards_root.pubkey().to_bytes(),
                &token_mint_pubkey.to_bytes(),
            ],
            &mplx_rewards::id(),
        );

        Self {
            deposit_authority,
            token_mint_pubkey,
            rewards_root,
            root_authority,
            mining_reward_pool,
        }
    }

    pub async fn initialize_pool(&self, context: &mut ProgramTestContext) -> BanksClientResult<()> {
        transfer(context, &self.root_authority.pubkey(), 10000000)
            .await
            .unwrap();

        // Initialize mining pool
        let tx = Transaction::new_signed_with_payer(
            &[
                mplx_rewards::instruction::initialize_root(
                    &mplx_rewards::id(),
                    &self.rewards_root.pubkey(),
                    &self.root_authority.pubkey(),
                ),
                mplx_rewards::instruction::initialize_pool(
                    &mplx_rewards::id(),
                    &self.rewards_root.pubkey(),
                    &self.mining_reward_pool,
                    &self.token_mint_pubkey,
                    &self.deposit_authority.pubkey(),
                    &self.root_authority.pubkey(),
                ),
            ],
            Some(&self.root_authority.pubkey()),
            &[&self.root_authority, &self.rewards_root],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn initialize_mining(
        &self,
        context: &mut ProgramTestContext,
        user: &Pubkey,
    ) -> Pubkey {
        let (mining_account, _) = Pubkey::find_program_address(
            &[
                b"mining".as_ref(),
                user.as_ref(),
                self.mining_reward_pool.as_ref(),
            ],
            &mplx_rewards::id(),
        );

        let tx = Transaction::new_signed_with_payer(
            &[mplx_rewards::instruction::initialize_mining(
                &mplx_rewards::id(),
                &self.mining_reward_pool,
                &mining_account,
                user,
                &context.payer.pubkey(),
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        mining_account
    }

    pub async fn deposit_mining(
        &self,
        context: &mut ProgramTestContext,
        user: &Pubkey,
        mining_account: &Pubkey,
        amount: u64,
    ) -> BanksClientResult<()> {
        let tx = Transaction::new_signed_with_payer(
            &[mplx_rewards::instruction::deposit_mining(
                &mplx_rewards::id(),
                &self.mining_reward_pool,
                &mining_account,
                user,
                &self.deposit_authority.pubkey(),
                amount,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.deposit_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn withdraw_mining(
        &self,
        context: &mut ProgramTestContext,
        user: &Pubkey,
        mining_account: &Pubkey,
        amount: u64,
    ) -> BanksClientResult<()> {
        let tx = Transaction::new_signed_with_payer(
            &[mplx_rewards::instruction::withdraw_mining(
                &mplx_rewards::id(),
                &self.mining_reward_pool,
                &mining_account,
                user,
                &self.deposit_authority.pubkey(),
                amount,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.deposit_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn add_vault(
        &self,
        context: &mut ProgramTestContext,
        fee_account: &Pubkey,
    ) -> Pubkey {
        let (vault_pubkey, _) = Pubkey::find_program_address(
            &[
                b"vault".as_ref(),
                self.mining_reward_pool.as_ref(),
                self.token_mint_pubkey.as_ref(),
            ],
            &mplx_rewards::id(),
        );

        let tx = Transaction::new_signed_with_payer(
            &[mplx_rewards::instruction::add_vault(
                &mplx_rewards::id(),
                &self.rewards_root.pubkey(),
                &self.mining_reward_pool,
                &self.token_mint_pubkey,
                &vault_pubkey,
                fee_account,
                &self.root_authority.pubkey(),
            )],
            Some(&self.root_authority.pubkey()),
            &[&self.root_authority],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        vault_pubkey
    }

    pub async fn fill_vault(
        &self,
        context: &mut ProgramTestContext,
        fee_account: &Pubkey,
        from: &Pubkey,
        amount: u64,
    ) -> BanksClientResult<()> {
        let (vault_pubkey, _) = Pubkey::find_program_address(
            &[
                b"vault".as_ref(),
                self.mining_reward_pool.as_ref(),
                self.token_mint_pubkey.as_ref(),
            ],
            &mplx_rewards::id(),
        );

        let tx = Transaction::new_signed_with_payer(
            &[mplx_rewards::instruction::fill_vault(
                &mplx_rewards::id(),
                &self.mining_reward_pool,
                &self.token_mint_pubkey,
                &vault_pubkey,
                fee_account,
                &context.payer.pubkey(),
                from,
                amount,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
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
        let (vault_pubkey, _) = Pubkey::find_program_address(
            &[
                b"vault".as_ref(),
                self.mining_reward_pool.as_ref(),
                self.token_mint_pubkey.as_ref(),
            ],
            &mplx_rewards::id(),
        );

        let tx = Transaction::new_signed_with_payer(
            &[mplx_rewards::instruction::claim(
                &mplx_rewards::id(),
                &self.mining_reward_pool,
                &self.token_mint_pubkey,
                &vault_pubkey,
                mining_account,
                &user.pubkey(),
                user_reward_token,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, user],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}

pub async fn account(context: &mut ProgramTestContext, pubkey: &Pubkey) -> Account {
    context
        .banks_client
        .get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty")
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

pub async fn presetup() -> (ProgramTestContext, Keypair) {
    let test = ProgramTest::new(
        "mplx_rewards",
        mplx_rewards::id(),
        processor!(mplx_rewards::processor::process_instruction),
    );

    let mut context = test.start_with_context().await;
    let payer_pubkey = context.payer.pubkey();

    // // TODO: check liquidity ming
    let liquidity_mint = Keypair::new();
    create_mint(&mut context, &liquidity_mint, &payer_pubkey)
        .await
        .unwrap();
    (context, liquidity_mint)
}
