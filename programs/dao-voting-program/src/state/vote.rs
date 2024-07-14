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
    pub bump: u8,
}

impl VoteState {
    /// Total size of the VoteState account in bytes
    pub const LEN: usize = 8 +                // Discriminator (added by Anchor)
        PUBKEY_LENGTH +    // owner: Pubkey
        U64_LENGTH +       // amount: u64
        ENUM_LENGTH +      // vote_type: VoteType (simple enum, 1 byte)
        U8_LENGTH; // bump: u8

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
