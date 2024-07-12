use crate::{constants::*, errors::DaoError};
use anchor_lang::prelude::*;

#[account]
pub struct Proposal {
    // A unique identifier for the proposal
    pub id: u64,

    // Name of the proposal (max 32 characters)
    pub name: String,

    // Brief description or GitHub gist URL
    // 72 bytes (39 bytes + / + 32 char ID)
    pub gist: String,

    // Type of the proposal (Bounty, Executable, or Vote)
    pub proposal: ProposalType,

    // Current status of the proposal
    pub result: ProposalStatus,

    // Minimum number of votes required for the proposal to pass
    pub quorum: u64,

    // Current number of votes cast
    pub votes: u64,

    // Slot at which the proposal expires
    pub expiry: u64,

    // Bump seed for the proposal's PDA
    pub bump: u8,
}

impl Proposal {
    // Total size of the Proposal account in bytes
    pub const LEN: usize = 8 + 32 + 72 + ENUM_LENGTH * 2 + U8_LENGTH * 2 + 3 * U64_LENGTH + U8_LENGTH;

    /// Initializes a new Proposal
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the proposal
    /// * `name` - Name of the proposal
    /// * `gist` - Brief description or GitHub gist URL
    /// * `proposal` - Type of the proposal
    /// * `quorum` - Minimum number of votes required
    /// * `expiry` - Duration in slots until the proposal expires
    /// * `bump` - Bump seed for the proposal's PDA
    ///
    /// # Errors
    ///
    /// Returns an error if the name or gist exceed their maximum lengths
    pub fn init(
        &mut self,
        id: u64,
        name: String,
        gist: String,
        proposal: ProposalType,
        quorum: u64,
        expiry: u64,
        bump: u8  
    ) -> Result<()> {
        require!(name.len() < 33, DaoError::InvalidName);
        require!(gist.len() < 73, DaoError::InvalidGist);
        self.id = id;
        self.proposal = proposal;
        self.name = name;
        self.gist = gist;
        self.result = ProposalStatus::Open;
        self.quorum = quorum;
        self.votes = 0;
        self.bump = bump;
        self.expiry = Clock::get()?.slot.checked_add(expiry).ok_or(DaoError::Overflow)?;
        Ok(())
    }

    /// Attempts to finalize the proposal based on votes and expiry
    pub fn try_finalize(&mut self) {
        if self.votes >= self.quorum && self.check_expiry().is_ok() {
            self.result = ProposalStatus::Succeeded
        } else if self.votes < self.quorum && self.check_expiry().is_err() {
            self.result = ProposalStatus::Failed
        }
    }

    /// Checks if the proposal has expired
    ///
    /// # Errors
    ///
    /// Returns an error if the current slot is greater than or equal to the expiry slot
    pub fn check_expiry(&mut self) -> Result<()> {
        require!(Clock::get()?.slot < self.expiry, DaoError::Expired);
        Ok(())
    }

    /// Checks if the proposal is still open
    ///
    /// # Errors
    ///
    /// Returns an error if the proposal is not in the Open status
    pub fn is_open(&mut self) -> Result<()> {
        require!(self.result == ProposalStatus::Open, DaoError::InvalidProposalStatus);
        Ok(())
    }

    /// Checks if the proposal has succeeded
    ///
    /// # Errors
    ///
    /// Returns an error if the proposal is not in the Succeeded status
    pub fn is_succeeded(&self) -> Result<()> {
        require!(self.result == ProposalStatus::Succeeded, DaoError::InvalidProposalStatus);
        Ok(())
    }

    /// Checks if the proposal has failed
    ///
    /// # Errors
    ///
    /// Returns an error if the proposal is not in the Failed status
    pub fn is_failed(&self) -> Result<()> {
        require!(self.result == ProposalStatus::Failed, DaoError::InvalidProposalStatus);
        Ok(())
    }

    /// Adds votes to the proposal
    ///
    /// # Arguments
    ///
    /// * `amount` - Number of votes to add
    ///
    /// # Errors
    ///
    /// Returns an error if adding votes results in an overflow
    pub fn add_vote(&mut self, amount: u64) -> Result<()> {
        self.votes = self.votes.checked_add(amount).ok_or(DaoError::Overflow)?;
        self.try_finalize();
        Ok(())
    }

    /// Removes votes from the proposal
    ///
    /// # Arguments
    ///
    /// * `amount` - Number of votes to remove
    ///
    /// # Errors
    ///
    /// Returns an error if removing votes results in an underflow
    pub fn remove_vote(&mut self, amount: u64) -> Result<()> {
        self.votes = self.votes.checked_sub(amount).ok_or(DaoError::Underflow)?;
        Ok(())
    }
}

/// Enum representing different types of proposals
#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq)]
pub enum ProposalType {
    Bounty(Pubkey, u64), // Pay an address some amount of SOL
    Executable,          // Sign some kind of instruction(s) with an accounts struct, etc
    Vote                 // We just want to know what people think. No money involved
}

/// Enum representing the current status of a proposal
#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProposalStatus {
    Open,      // The proposal is active and accepting votes
    Succeeded, // The proposal has passed (met quorum and not expired)
    Failed     // The proposal has failed (didn't meet quorum or expired)
}