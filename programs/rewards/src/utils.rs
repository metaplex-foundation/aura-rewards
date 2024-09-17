//! Arbitrary auxilliary functions
use std::iter::Enumerate;

use crate::{
    asserts::{assert_account_key, assert_pubkey_eq},
    error::MplxRewardsError,
    state::WrappedImmutableMining,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    clock::{Clock, SECONDS_PER_DAY},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

/// Generates mining address
pub fn find_mining_program_address(
    program_id: &Pubkey,
    mining_owner: &Pubkey,
    reward_pool: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "mining".as_bytes(),
            &mining_owner.to_bytes(),
            &reward_pool.to_bytes(),
        ],
        program_id,
    )
}

/// Generates mining address with a specific seed
pub fn get_mining_address(
    program_id: &Pubkey,
    mining_owner: &Pubkey,
    reward_pool: &Pubkey,
    mining_bump: u8,
) -> Result<Pubkey, ProgramError> {
    Pubkey::create_program_address(
        &[
            b"mining".as_ref(),
            mining_owner.as_ref(),
            reward_pool.as_ref(),
            &[mining_bump],
        ],
        program_id,
    )
    .map_err(|_| MplxRewardsError::AccountDerivationAddresFailed.into())
}

/// Generates vault address
pub fn find_vault_program_address(
    program_id: &Pubkey,
    reward_pool: &Pubkey,
    reward_mint: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "vault".as_bytes(),
            &reward_pool.to_bytes(),
            &reward_mint.to_bytes(),
        ],
        program_id,
    )
}

/// Create account
pub fn create_account<'a, S: Pack>(
    program_id: &Pubkey,
    from: AccountInfo<'a>,
    to: AccountInfo<'a>,
    signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
    let rent = Rent::get()?;

    let ix = system_instruction::create_account(
        from.key,
        to.key,
        rent.minimum_balance(S::LEN),
        S::LEN as u64,
        program_id,
    );

    invoke_signed(&ix, &[from, to], signers_seeds)
}

/// Initialize SPL account instruction.
pub fn initialize_account<'a>(
    account: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    rent: AccountInfo<'a>,
) -> ProgramResult {
    let ix = spl_token::instruction::initialize_account(
        &spl_token::id(),
        account.key,
        mint.key,
        authority.key,
    )?;

    invoke(&ix, &[account, mint, authority, rent])
}

/// SPL transfer instruction.
pub fn spl_transfer<'a>(
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    amount: u64,
    signers_seeds: &[&[&[u8]]],
) -> Result<(), ProgramError> {
    let ix = spl_token::instruction::transfer(
        &spl_token::id(),
        source.key,
        destination.key,
        authority.key,
        &[],
        amount,
    )?;

    invoke_signed(&ix, &[source, destination, authority], signers_seeds)
}

pub fn get_delegate_mining<'a, 'b>(
    delegate_mining: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
) -> Result<Option<&'a AccountInfo<'b>>, ProgramError> {
    if mining.key != delegate_mining.key {
        Ok(Some(delegate_mining))
    } else {
        // None means delegate_mining is the same as mining
        Ok(None)
    }
}

// Verifies that delegate mining has been derived from the specified pool and mining owner
pub fn vefiry_delegate_mining(
    delegate_mining: Option<&AccountInfo>,
    mining_owner: &Pubkey,
    program_id: &Pubkey,
    reward_pool: &Pubkey,
) -> ProgramResult {
    if let Some(delegate_mining_info) = delegate_mining {
        let delegate_mining_data = &mut delegate_mining_info.data.borrow_mut();
        let wrapped_delegate_mining = WrappedImmutableMining::from_bytes(delegate_mining_data)?;

        let mining_pubkey = get_mining_address(
            program_id,
            mining_owner,
            reward_pool,
            wrapped_delegate_mining.mining.bump,
        )?;
        assert_pubkey_eq(mining_owner, &wrapped_delegate_mining.mining.owner)?;
        assert_account_key(delegate_mining_info, &mining_pubkey)?;
    }

    Ok(())
}

/// Helper for parsing accounts with arbitrary input conditions
pub struct AccountLoader {}

impl AccountLoader {
    /// Checks that account is not initilized (it's pubkey is empty)
    pub fn next_uninitialized<'a, 'b, I: Iterator<Item = &'a AccountInfo<'b>>>(
        iter: &mut Enumerate<I>,
    ) -> Result<I::Item, ProgramError> {
        let (idx, acc) = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;

        acc.data
            .borrow()
            .iter()
            .all(|&x| x == 0)
            .then_some(acc)
            .ok_or_else(|| {
                msg!("Account #{}:{} is already initialized", idx, acc.key);
                ProgramError::AccountAlreadyInitialized
            })
    }

    /// Checks if the next account has an owner with the specified address
    pub fn next_with_owner<'a, 'b, I: Iterator<Item = &'a AccountInfo<'b>>>(
        iter: &mut Enumerate<I>,
        owner: &Pubkey,
    ) -> Result<I::Item, ProgramError> {
        let (idx, acc) = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
        if acc.owner.eq(owner) {
            return Ok(acc);
        }

        msg!(
            "Account #{}:{} owner error. Got {} Expected {}",
            idx,
            acc.key,
            acc.owner,
            owner
        );
        Err(MplxRewardsError::InvalidAccountOwner.into())
    }

    /// Checks whether next account matches a given key
    pub fn next_with_key<'a, 'b, I: Iterator<Item = &'a AccountInfo<'b>>>(
        iter: &mut Enumerate<I>,
        key: &Pubkey,
    ) -> Result<I::Item, ProgramError> {
        let (idx, acc) = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
        if acc.key.eq(key) {
            return Ok(acc);
        }

        msg!(
            "Account #{}:{} assert error. Expected {}",
            idx,
            acc.key,
            key
        );
        Err(ProgramError::InvalidArgument)
    }

    /// Checks if next account is a signer
    pub fn next_signer<'a, 'b, I: Iterator<Item = &'a AccountInfo<'b>>>(
        iter: &mut Enumerate<I>,
    ) -> Result<I::Item, ProgramError> {
        let (idx, acc) = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
        if acc.is_signer {
            return Ok(acc);
        }

        msg!("Account #{}:{} missing signature", idx, acc.key,);
        Err(ProgramError::MissingRequiredSignature)
    }

    /// Load the account without any checks
    pub fn next_unchecked<'a, 'b, I: Iterator<Item = &'a AccountInfo<'b>>>(
        iter: &mut Enumerate<I>,
    ) -> Result<I::Item, ProgramError> {
        let (_, acc) = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
        Ok(acc)
    }

    /// Shows true when an iterator has more elements
    pub fn has_more<I: Iterator>(iter: &Enumerate<I>) -> bool {
        let (remaining_len, _) = iter.size_hint();
        remaining_len > 0
    }
}

/// LockupPeriod is used to define the time during which the lockup will recieve full reward
#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum LockupPeriod {
    /// Unreachable option
    None,
    Test,
    /// Unlimited lockup period.
    Flex,
    /// Three months
    ThreeMonths,
    /// SixMonths
    SixMonths,
    /// OneYear
    OneYear,
}

impl LockupPeriod {
    /// Converts LockupPeriod into the Multiplier
    /// which will be used in rewards calculations
    pub fn multiplier(&self) -> u64 {
        match self {
            LockupPeriod::None => 0,
            LockupPeriod::ThreeMonths => 2,
            LockupPeriod::SixMonths => 4,
            LockupPeriod::OneYear => 6,
            LockupPeriod::Flex => 1,
            LockupPeriod::Test => 1,
        }
    }

    /// Calculates the time when a lockup should expire
    pub fn end_timestamp(&self, start_ts: u64) -> Result<u64, MplxRewardsError> {
        // conversion should be unfailable because negative timestamp means the ts is earlier than 1970y
        let beginning_of_the_day = start_ts - (start_ts % SECONDS_PER_DAY);

        match self {
            LockupPeriod::None => Err(MplxRewardsError::InvalidLockupPeriod),
            LockupPeriod::ThreeMonths => Ok(beginning_of_the_day + SECONDS_PER_DAY * 90),
            LockupPeriod::SixMonths => Ok(beginning_of_the_day + SECONDS_PER_DAY * 180),
            LockupPeriod::OneYear => Ok(beginning_of_the_day + SECONDS_PER_DAY * 365),
            LockupPeriod::Flex => Ok(beginning_of_the_day + SECONDS_PER_DAY * 5),
            LockupPeriod::Test => Ok(beginning_of_the_day + 120),
        }
    }

    /// Return number of days plain numbers to make them appliable for the self.weighted_stake_diff
    pub fn days(&self) -> Result<u64, MplxRewardsError> {
        match self {
            LockupPeriod::None => Err(MplxRewardsError::InvalidLockupPeriod),
            LockupPeriod::ThreeMonths => Ok(90),
            LockupPeriod::SixMonths => Ok(180),
            LockupPeriod::OneYear => Ok(365),
            LockupPeriod::Flex => Ok(5),
            LockupPeriod::Test => Ok(0),
        }
    }
}

/// Get current unix time
#[inline]
pub fn get_curr_unix_ts() -> u64 {
    // Conversion must be save because negative values
    // in unix means the date is earlier than 1970y
    Clock::get().unwrap().unix_timestamp as u64
}

pub(crate) trait SafeArithmeticOperations
where
    Self: std::marker::Sized,
{
    fn safe_sub(&self, amount: Self) -> Result<Self, MplxRewardsError>;
    fn safe_add(&self, amount: Self) -> Result<Self, MplxRewardsError>;
    fn safe_mul(&self, amount: Self) -> Result<Self, MplxRewardsError>;
    fn safe_div(&self, amount: Self) -> Result<Self, MplxRewardsError>;
}

impl SafeArithmeticOperations for u64 {
    fn safe_sub(&self, amount: u64) -> Result<u64, MplxRewardsError> {
        self.checked_sub(amount)
            .ok_or(MplxRewardsError::MathOverflow)
    }

    fn safe_add(&self, amount: u64) -> Result<u64, MplxRewardsError> {
        self.checked_add(amount)
            .ok_or(MplxRewardsError::MathOverflow)
    }

    fn safe_mul(&self, amount: u64) -> Result<u64, MplxRewardsError> {
        self.checked_mul(amount)
            .ok_or(MplxRewardsError::MathOverflow)
    }

    fn safe_div(&self, amount: u64) -> Result<u64, MplxRewardsError> {
        self.checked_div(amount)
            .ok_or(MplxRewardsError::MathOverflow)
    }
}

impl SafeArithmeticOperations for u128 {
    fn safe_sub(&self, amount: u128) -> Result<u128, MplxRewardsError> {
        self.checked_sub(amount)
            .ok_or(MplxRewardsError::MathOverflow)
    }

    fn safe_add(&self, amount: u128) -> Result<u128, MplxRewardsError> {
        self.checked_add(amount)
            .ok_or(MplxRewardsError::MathOverflow)
    }

    fn safe_mul(&self, amount: u128) -> Result<u128, MplxRewardsError> {
        self.checked_mul(amount)
            .ok_or(MplxRewardsError::MathOverflow)
    }

    fn safe_div(&self, amount: u128) -> Result<u128, MplxRewardsError> {
        self.checked_div(amount)
            .ok_or(MplxRewardsError::MathOverflow)
    }
}
