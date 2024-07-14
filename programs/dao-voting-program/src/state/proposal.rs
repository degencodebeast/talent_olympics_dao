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

    // The total number of 'yes' votes for the proposal
    pub yes_votes: u64,

    // The total number of 'no' votes for the proposal
    pub no_votes: u64,

    // The total number of 'abstain' votes for the proposal
    pub abstain_votes: u64,

    // Bump seed for the proposal's PDA
    pub bump: u8,
}
// Discriminator: 8 bytes
// id: u64 = 8 bytes
// name: String (max 32 characters) = 4 bytes (for length) + 32 bytes = 36 bytes
// gist: String (max 72 characters) = 4 bytes (for length) + 72 bytes = 76 bytes
// proposal: ProposalType (enum) = 1 byte (for discriminator) + 8 bytes (for largest variant) = 9 bytes
// result: ProposalStatus (enum) = 1 byte
// quorum: u64 = 8 bytes
// votes: u64 = 8 bytes
// expiry: u64 = 8 bytes
// yes_votes: u64 = 8 bytes
// no_votes: u64 = 8 bytes
// abstain_votes: u64 = 8 bytes
// bump: u8 = 1 byte
impl Proposal {
    // Total size of the Proposal account in bytes
    //pub const LEN: usize = 8 + 32 + 72 + ENUM_LENGTH * 2 + U8_LENGTH * 2 + 7 * U64_LENGTH + U8_LENGTH;
    pub const LEN: usize = 187;

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
        bump: u8,
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
        self.yes_votes = 0;
        self.no_votes = 0;
        self.abstain_votes = 0;
        self.bump = bump;
        self.expiry = Clock::get()?
            .slot
            .checked_add(expiry)
            .ok_or(DaoError::Overflow)?;
        Ok(())
    }

    /// Attempts to finalize the proposal based on votes and expiry
    pub fn try_finalize(&mut self) -> Result<()> {
        // First, check if the proposal has already been finalized
        if self.result != ProposalStatus::Open {
            return Ok(()); // Proposal has already been finalized
        }

        // Check if the proposal has expired
        let has_expired = self.check_expiry().is_err();

        // Calculate total votes
        let total_votes = self.yes_votes + self.no_votes;

        // Determine the result of the proposal
        self.result = if total_votes >= self.quorum {
            // Quorum reached, decision can be made based on vote count
            if self.yes_votes > self.no_votes {
                ProposalStatus::Succeeded // More 'yes' votes, proposal passes
            } else {
                ProposalStatus::Failed // Equal or more 'no' votes, proposal fails
            }
        } else if has_expired {
            // Quorum not reached and voting period has ended
            ProposalStatus::Failed // Proposal fails due to insufficient participation
        } else {
            // Quorum not reached but voting period is still open
            ProposalStatus::Open // Proposal remains open for more votes
        };

        Ok(())
    }

    // pub fn try_finalize(&mut self) {
    //     if self.yes_votes >= self.quorum && self.yes_votes > self.no_votes && self.check_expiry().is_ok() {
    //         self.result = ProposalStatus::Succeeded
    //     } else if self.votes < self.quorum && self.check_expiry().is_err() {
    //         self.result = ProposalStatus::Failed
    //     }
    // }

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
        require!(
            self.result == ProposalStatus::Open,
            DaoError::InvalidProposalStatus
        );
        Ok(())
    }

    /// Checks if the proposal has succeeded
    ///
    /// # Errors
    ///
    /// Returns an error if the proposal is not in the Succeeded status
    pub fn is_succeeded(&self) -> Result<()> {
        require!(
            self.result == ProposalStatus::Succeeded,
            DaoError::InvalidProposalStatus
        );
        Ok(())
    }

    /// Checks if the proposal has failed
    ///
    /// # Errors
    ///
    /// Returns an error if the proposal is not in the Failed status
    pub fn is_failed(&self) -> Result<()> {
        require!(
            self.result == ProposalStatus::Failed,
            DaoError::InvalidProposalStatus
        );
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

    pub fn add_vote(&mut self, amount: u64, vote_type: VoteType) -> Result<()> {
        self.votes = self.votes.checked_add(amount).ok_or(DaoError::Overflow)?;
        match vote_type {
            VoteType::Yes => {
                self.yes_votes = self
                    .yes_votes
                    .checked_add(amount)
                    .ok_or(DaoError::Overflow)?
            }
            VoteType::No => {
                self.no_votes = self
                    .no_votes
                    .checked_add(amount)
                    .ok_or(DaoError::Overflow)?
            }
            VoteType::Abstain => {
                self.abstain_votes = self
                    .abstain_votes
                    .checked_add(amount)
                    .ok_or(DaoError::Overflow)?
            }
        }
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
    pub fn remove_vote(&mut self, amount: u64, vote_type: VoteType) -> Result<()> {
        self.votes = self.votes.checked_sub(amount).ok_or(DaoError::Underflow)?;
        match vote_type {
            VoteType::Yes => {
                self.yes_votes = self
                    .yes_votes
                    .checked_sub(amount)
                    .ok_or(DaoError::Underflow)?
            }
            VoteType::No => {
                self.no_votes = self
                    .no_votes
                    .checked_sub(amount)
                    .ok_or(DaoError::Underflow)?
            }
            VoteType::Abstain => {
                self.abstain_votes = self
                    .abstain_votes
                    .checked_sub(amount)
                    .ok_or(DaoError::Underflow)?
            }
        }
        Ok(())
    }

    pub fn get_results(&self) -> ProposalResults {
        ProposalResults {
            yes_votes: self.yes_votes,
            no_votes: self.no_votes,
            abstain_votes: self.abstain_votes,
            status: self.result,
            total_votes: self.votes,
            quorum: self.quorum,
        }
    }
}

/// Enum representing different types of proposals
#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq)]
pub enum ProposalType {
    Bounty(Pubkey, u64), // Pay an address some amount of SOL
    Executable,          // Sign some kind of instruction(s) with an accounts struct, etc
    Vote,                // We just want to know what people think. No money involved
}

/// Enum representing the current status of a proposal
#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProposalStatus {
    Open,      // The proposal is active and accepting votes
    Succeeded, // The proposal has passed (met quorum and not expired)
    Failed,    // The proposal has failed (didn't meet quorum or expired)
}

/// Enum representing the type of vote
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Copy, Eq)]
pub enum VoteType {
    Yes,
    No,
    Abstain,
}

/// Struct representing the results of a proposal
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ProposalResults {
    pub yes_votes: u64,
    pub no_votes: u64,
    pub abstain_votes: u64,
    pub status: ProposalStatus,
    pub total_votes: u64,
    pub quorum: u64,
}
