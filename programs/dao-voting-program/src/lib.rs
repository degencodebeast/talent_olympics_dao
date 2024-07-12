use anchor_lang::prelude::*;

mod instructions;
use instructions::*;
mod state;
mod errors;
mod constants;

declare_id!("HScCa2Qkqn5DRFDXySiwhtTMNJs87hdo8DjsBxvBfyn6");

#[program]
pub mod dao_voting_program {
    use crate::{errors::DaoError, state::ProposalType};
    
    use super::*;

    // Instantiate a new DAO
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
    
    // // Vote on a proposal
    // pub fn vote(ctx: Context<Vote>, amount: u64) -> Result<()> {
    //     // Increment total number of votes in the proposal
    //     ctx.accounts.vote(amount, *ctx.bumps.get("vote").ok_or(DaoError::BumpError)?)
    // }

}

#[derive(Accounts)]
pub struct Initialize {}
