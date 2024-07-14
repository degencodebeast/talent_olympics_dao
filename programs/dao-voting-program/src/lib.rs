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

    use state::ProposalResults;

    use crate::{errors::DaoError, state::ProposalType};

    use super::*;

    // Initialize a DAO

    pub fn initialize(
        ctx: Context<Initialize>,
        seed: u64,
        issue_price: u64,
        issue_amount: u64,
        proposal_fee: u64,
        max_supply: u64,
        min_quorum: u64,
        max_expiry: u64,
    ) -> Result<()> {
        ctx.accounts.init(
            seed,
            &ctx.bumps,
            issue_price,
            issue_amount,
            proposal_fee,
            max_supply,
            min_quorum,
            max_expiry,
        )
    }

    // Vote on a proposal
    pub fn vote(ctx: Context<Vote>, amount: u64, vote_type: VoteType) -> Result<()> {
        // Increment total number of votes in the proposal

        let bump = *ctx.bumps.get("vote").ok_or(DaoError::BumpError)?;

        ctx.accounts.vote(amount, vote_type, bump)
    }

    // Initialize a stake account for adding DAO tokens
    pub fn init_stake(ctx: Context<InitializeStake>) -> Result<()> {
        // Create a stake account
        ctx.accounts.init(&ctx.bumps)
    }

    // Stake get proposal results
    pub fn get_proposal_results(ctx: Context<GetProposalResults>) -> Result<ProposalResults> {
        ctx.accounts.get_results()
    }

    // Stake DAO tokens
    pub fn stake_tokens(ctx: Context<Stake>, amount: u64) -> Result<()> {
        // Deposit tokens, add stake
        ctx.accounts.deposit_tokens(amount)
    }

    // Close a stake account when you're done with it
    pub fn close_stake(ctx: Context<CloseStakeAccount>) -> Result<()> {
        // Create a stake account
        ctx.accounts.cleanup()
    }

    // Unstake DAO tokens
    pub fn unstake_tokens(ctx: Context<Stake>, amount: u64) -> Result<()> {
        // Withdraw tokens, remove stake
        ctx.accounts.withdraw_tokens(amount)
    }
}
