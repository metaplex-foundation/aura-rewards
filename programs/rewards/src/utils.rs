//! Arbitrary auxilliary functions
use std::iter::Enumerate;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::clock::Clock;
use solana_program::clock::SECONDS_PER_DAY;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use crate::error::MplxRewardsError;

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

/// Generates reward pool address
pub fn find_reward_pool_program_address(
    program_id: &Pubkey,
    authority_account: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &["reward_pool".as_bytes(), &authority_account.to_bytes()],
        program_id,
    )
}

/// Trait that verifies whether an acount is initialized
pub trait Uninitialized {
    /// Is uninitialized
    fn is_uninitialized(&self) -> bool;
}

/// Assert signer.
pub fn assert_signer(account: &AccountInfo) -> ProgramResult {
    if account.is_signer {
        return Ok(());
    }

    Err(ProgramError::MissingRequiredSignature)
}

/// Assert initilialized
pub fn assert_initialized<T: IsInitialized>(account: &T) -> ProgramResult {
    if account.is_initialized() {
        Ok(())
    } else {
        Err(ProgramError::UninitializedAccount)
    }
}

/// Assert unitilialized
pub fn assert_uninitialized<T: Uninitialized>(account: &T) -> ProgramResult {
    if account.is_uninitialized() {
        Ok(())
    } else {
        Err(ProgramError::AccountAlreadyInitialized)
    }
}

/// Assert owned by
pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account.owner == owner {
        Ok(())
    } else {
        msg!(
            "Assert {} owner error. Got {} Expected {}",
            *account.key,
            *account.owner,
            *owner
        );
        Err(MplxRewardsError::InvalidAccountOwner.into())
    }
}

/// Assert account key
pub fn assert_account_key(account_info: &AccountInfo, key: &Pubkey) -> ProgramResult {
    if *account_info.key == *key {
        Ok(())
    } else {
        msg!(
            "Assert account error. Got {} Expected {}",
            *account_info.key,
            *key
        );
        Err(ProgramError::InvalidArgument)
    }
}

/// Assert rent exempt
pub fn assert_rent_exempt(account_info: &AccountInfo) -> ProgramResult {
    let rent = Rent::get()?;

    if rent.is_exempt(account_info.lamports(), account_info.data_len()) {
        Ok(())
    } else {
        msg!(&rent.minimum_balance(account_info.data_len()).to_string());
        Err(ProgramError::AccountNotRentExempt)
    }
}

/// Assert a non-zero amount
pub fn assert_non_zero_amount(amount: u64) -> ProgramResult {
    if amount == 0 {
        return Err(MplxRewardsError::ZeroAmount.into());
    }

    Ok(())
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

/// Helper for parsing accounts with arbitrary input conditions
pub struct AccountLoader {}

impl AccountLoader {
    /// Checks that account is not initilized (it's pubkey is empty)
    pub fn next_uninitialized<'a, 'b, I: Iterator<Item = &'a AccountInfo<'b>>>(
        iter: &mut Enumerate<I>,
    ) -> Result<I::Item, ProgramError> {
        let (idx, acc) = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;

        let AccountInfo {
            key,
            lamports,
            data,
            owner,
            ..
        } = acc;

        if **lamports.borrow() == 0 && data.borrow().is_empty() && *owner == &Pubkey::default() {
            return Ok(acc);
        }

        msg!("Account #{}:{} already initialized", idx, key,);
        Err(ProgramError::AccountAlreadyInitialized)
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

    /// Checks if account is initialized and then checks it's owner
    pub fn next_optional<'a, 'b, I: Iterator<Item = &'a AccountInfo<'b>>>(
        iter: &mut Enumerate<I>,
        owner: &Pubkey,
    ) -> Result<I::Item, ProgramError> {
        let (idx, acc) = iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
        if acc.owner.eq(&Pubkey::default()) {
            return Ok(acc);
        }

        if acc.owner.eq(owner) {
            return Ok(acc);
        }

        msg!(
            "Account #{}:{} owner error. Got {} Expected unitialized or {}",
            idx,
            acc.key,
            acc.owner,
            owner
        );
        Err(MplxRewardsError::InvalidAccountOwner.into())
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
#[repr(u8)]
#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum LockupPeriod {
    /// Unreachable option
    None,
    /// Three months
    ThreeMonths,
    /// SixMonths
    SixMonths,
    /// OneYear
    OneYear,
    /// Unlimited lockup period.
    Flex,
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
        }
    }

    /// Calculates the time when a lockup should expire
    pub fn end_timestamp(&self, start_ts: u64) -> Result<u64, MplxRewardsError> {
        // conversion should be unfailable because negative timestamp means the ts is earlier than 1970y
        let beginning_of_the_day = start_ts - (start_ts % SECONDS_PER_DAY);

        match self {
            LockupPeriod::None | LockupPeriod::Flex => Err(MplxRewardsError::InvalidLockupPeriod),
            LockupPeriod::ThreeMonths => Ok(beginning_of_the_day + SECONDS_PER_DAY * 90),
            LockupPeriod::SixMonths => Ok(beginning_of_the_day + SECONDS_PER_DAY * 180),
            LockupPeriod::OneYear => Ok(beginning_of_the_day + SECONDS_PER_DAY * 365),
        }
    }

    /// Return number of days plain numbers to make them appliable for the self.weighted_stake_diff
    pub fn days(&self) -> Result<u64, MplxRewardsError> {
        match self {
            LockupPeriod::None => Err(MplxRewardsError::InvalidLockupPeriod),
            LockupPeriod::ThreeMonths => Ok(90),
            LockupPeriod::SixMonths => Ok(180),
            LockupPeriod::OneYear => Ok(365),
            LockupPeriod::Flex => Ok(0),
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
