use anchor_lang::prelude::*;

mod instructions;
use instructions::*;
mod constants;
mod errors;
mod state;
use crate::state::VoteType;

declare_id!("HScCa2Qkqn5DRFDXySiwhtTMNJs87hdo8DjsBxvBfyn6");

#[program]
pub mod dao_voting_program {

    use crate::{errors::DaoError, state::ProposalType};

    use super::*;

    // Instantiate a new DAO
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    // Vote on a proposal
    pub fn vote(ctx: Context<Vote>, amount: u64, vote_type: VoteType) -> Result<()> {
        // Increment total number of votes in the proposal

        let bump = *ctx.bumps.get("vote").ok_or(DaoError::BumpError)?;
        
        ctx.accounts.vote(
            amount,
            vote_type,
            bump,
        )
    }

    // pub fn get_proposal_results(ctx: Context<GetProposalResults>) -> Result<ProposalResults> {
    //     ctx.accounts.get_results()
    // }
}

#[derive(Accounts)]
pub struct Initialize {}
