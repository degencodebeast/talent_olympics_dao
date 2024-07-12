use crate::{constants::*, errors::DaoError};
use anchor_lang::prelude::*;

// The StakeState account struct represents the staking state of a user in the DAO
#[account]
pub struct StakeState {
    // The public key of the account owner (staker)
    pub owner: Pubkey,

    // The total amount of tokens staked by this owner
    pub amount: u64,

    // The number of accounts (likely voting accounts) associated with this stake
    pub accounts: u64,

    // The slot at which this stake state was last updated
    pub updated: u64,

    // Bump seed for the vault's Program Derived Address (PDA)
    pub vault_bump: u8,

    // Bump seed for the authority's Program Derived Address (PDA)
    pub auth_bump: u8,

    // Bump seed for this stake state's Program Derived Address (PDA)
    pub state_bump: u8,
}

impl StakeState {
    // Constant representing the total size of the StakeState account in bytes
    // 8 (discriminator) + PUBKEY_L (public key length) + 3 * U64_LENGTH (for amount, accounts, updated)
    // + 3 * U8_LENGTH (for vault_bump, auth_bump, state_bump)
    pub const LEN: usize = 8 + PUBKEY_LENGTH + 3 * U64_LENGTH + 3 * U8_LENGTH;

    // Initializes a new StakeState account
    pub fn init(
        &mut self,  
        owner: Pubkey,
        state_bump: u8,
        vault_bump: u8,
        auth_bump: u8
    ) -> Result<()> {
        self.owner = owner;
        self.amount = 0;
        self.accounts = 0;
        self.state_bump = state_bump;
        self.vault_bump = vault_bump;
        self.auth_bump = auth_bump;
        self.update()
    }

    // Increases the staked amount
    pub fn stake(
        &mut self,
        amount: u64
    ) -> Result<()> {
        self.amount = self.amount.checked_add(amount).ok_or(DaoError::Overflow)?;
        self.update()
    }

    // Decreases the staked amount, with additional checks
    pub fn unstake(
        &mut self,
        amount: u64
    ) -> Result<()> {
        self.check_accounts()?;
        self.check_slot()?; // Don't allow staking and unstaking in the same slot
        self.amount = self.amount.checked_sub(amount).ok_or(DaoError::Underflow)?;
        self.update()
    }

    // Increments the number of associated accounts
    pub fn add_account(&mut self) -> Result<()> {
        self.accounts = self.accounts.checked_add(1).ok_or(DaoError::Overflow)?;
        Ok(())
    }

    // Decrements the number of associated accounts
    pub fn remove_account(&mut self) -> Result<()> {
        self.accounts = self.accounts.checked_sub(1).ok_or(DaoError::Underflow)?;
        Ok(())
    }

    // Updates the 'updated' field with the current slot
    pub fn update(&mut self) -> Result<()> {
        self.updated = Clock::get()?.slot;
        Ok(())
    }

    // Ensures that unstaking doesn't occur in the same slot as a previous action
    pub fn check_slot(&mut self) -> Result<()> {
        require!(self.updated < Clock::get()?.slot, DaoError::InvalidSlot);
        Ok(())
    }    

    // Ensures that the user doesn't have any open accounts before unstaking
    pub fn check_accounts(&mut self) -> Result<()> {
        require!(self.accounts == 0, DaoError::AccountsOpen);
        Ok(())
    }

    // Verifies that the staked amount is greater than zero
    pub fn check_stake(&mut self) -> Result<()> {
        require!(self.amount > 0, DaoError::InsufficientStake);
        Ok(())
    }

    // Ensures that the staked amount is greater than or equal to a specified amount
    pub fn check_stake_amount(&mut self, amount: u64) -> Result<()> {
        require!(self.amount >= amount, DaoError::InsufficientStake);
        Ok(())
    }
}