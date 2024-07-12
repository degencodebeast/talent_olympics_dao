use crate::{constants::*, errors::DaoError};
use anchor_lang::prelude::*;

use super::VoteType;

// The VoteState account struct represents the voting state of a user for a specific proposal
#[account]
pub struct VoteState {
    // The public key of the account owner (voter)
    pub owner: Pubkey,

    // The amount of votes cast by this owner
    pub amount: u64,

    /// Enum representing the type of vote
    pub vote_type: VoteType,

    // Bump seed for the vote state's Program Derived Address (PDA)
    pub bump: u8
}

impl VoteState {
    // Constant representing the total size of the VoteState account in bytes
    // 8 (discriminator) + PUBKEY_L (public key length) + U64_L (u64 length) + U8_L (u8 length)
    pub const LEN: usize = 8 + PUBKEY_LENGTH + U64_LENGTH + U8_LENGTH + 1 + 1;

    // Initializes a new VoteState account
    pub fn init(
        &mut self,
        owner: Pubkey,
        amount: u64,
        vote_type: VoteType,
        bump: u8,
    ) -> Result<()> {
        self.owner = owner;
        self.amount = amount;
        self.vote_type = vote_type;
        self.bump = bump;
        Ok(())
    }
}