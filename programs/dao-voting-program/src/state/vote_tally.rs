use crate::{constants::*, errors::DaoError};
use anchor_lang::prelude::*;

// The VoteTally account struct represents the tally of votes for a specific proposal
#[account]
pub struct VoteTally {
    // The unique identifier of the proposal
    pub proposal_id: u64,

    // The total number of 'yes' votes for the proposal
    pub yes_votes: u64,

    // The total number of 'no' votes for the proposal
    pub no_votes: u64,

    // The total number of 'abstain' votes for the proposal
    pub abstain_votes: u64,

    // Bump seed for the vote tally's Program Derived Address (PDA)
    pub bump: u8,
}

impl VoteTally {
    // Constant representing the total size of the VoteTally account in bytes
    // 8 (discriminator) + 4 * U64_LENGTH (for proposal_id, yes_votes, no_votes, abstain_votes) + U8_LENGTH (for bump)
    pub const LEN: usize = 8 + 4 * U64_LENGTH + U8_LENGTH;

    // Initializes a new VoteTally account
    pub fn init(
        &mut self,
        proposal_id: u64,
        bump: u8,
    ) -> Result<()> {
        self.proposal_id = proposal_id;
        self.yes_votes = 0;
        self.no_votes = 0;
        self.abstain_votes = 0;
        self.bump = bump;
        Ok(())
    }

    // Adds votes to the tally
    pub fn add_votes(&mut self, yes: u64, no: u64, abstain: u64) -> Result<()> {
        self.yes_votes = self.yes_votes.checked_add(yes).ok_or(DaoError::Overflow)?;
        self.no_votes = self.no_votes.checked_add(no).ok_or(DaoError::Overflow)?;
        self.abstain_votes = self.abstain_votes.checked_add(abstain).ok_or(DaoError::Overflow)?;
        Ok(())
    }

    // Removes votes from the tally
    pub fn remove_votes(&mut self, yes: u64, no: u64, abstain: u64) -> Result<()> {
        self.yes_votes = self.yes_votes.checked_sub(yes).ok_or(DaoError::Underflow)?;
        self.no_votes = self.no_votes.checked_sub(no).ok_or(DaoError::Underflow)?;
        self.abstain_votes = self.abstain_votes.checked_sub(abstain).ok_or(DaoError::Underflow)?;
        Ok(())
    }

    // Returns the total number of votes
    pub fn total_votes(&self) -> u64 {
        self.yes_votes.saturating_add(self.no_votes).saturating_add(self.abstain_votes)
    }

    // Checks if the proposal has passed based on a simple majority
    pub fn has_passed(&self) -> bool {
        self.yes_votes > self.no_votes
    }
}