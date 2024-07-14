use anchor_lang::prelude::*;

mod instructions;
use instructions::*;
mod constants;
mod errors;
mod state;
use crate::state::VoteType;

use crate::{errors::DaoError, state::ProposalType};

declare_id!("HScCa2Qkqn5DRFDXySiwhtTMNJs87hdo8DjsBxvBfyn6");

#[program]
pub mod dao_voting_program {

    use state::{MemberStateView, ProposalResults};

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

    // Initialize a stake account for adding DAO tokens
    pub fn init_stake(ctx: Context<InitializeStake>) -> Result<()> {
        // Create a stake account
        ctx.accounts.init(&ctx.bumps)
    }

    // Create a proposal
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        id: u64,
        name: String,
        description: String,
        proposal_type: ProposalType,
        threshold: u64,
        amount: u64,
        //data: Vec<u8>,
    ) -> Result<()> {
        // Pay a proposal fee to DAO treasury
        ctx.accounts.pay_proposal_fee()?;

        // Ensure user has actually got tokens staked and create a new proposal
        ctx.accounts.create_proposal(
            id,
            name,
            description,
            proposal_type,
            threshold,
            amount,
            *ctx.bumps.get("proposal").ok_or(DaoError::BumpError)?,
        )
    }

    // Stake get proposal results
    pub fn get_proposal_results(ctx: Context<GetProposalResults>) -> Result<ProposalResults> {
        ctx.accounts.get_results()
    }

    // Stake DAO tokens
    pub fn stake_tokens(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let bump = *ctx.bumps.get("member").ok_or(DaoError::BumpError)?;
        // Deposit tokens, add stake
        ctx.accounts.deposit_tokens(amount, bump)
    }

    // Vote on a proposal
    pub fn vote(ctx: Context<Vote>, amount: u64, vote_type: VoteType) -> Result<()> {
        // Increment total number of votes in the proposal

        let bump = *ctx.bumps.get("vote").ok_or(DaoError::BumpError)?;

        ctx.accounts.vote(amount, vote_type, bump)
    }

    // Close a stake account when you're done with it
    pub fn close_stake_account(ctx: Context<CloseStakeAccount>) -> Result<()> {
        // Create a stake account
        ctx.accounts.cleanup()
    }

    // Unstake DAO tokens
    pub fn unstake_tokens(ctx: Context<Stake>, amount: u64) -> Result<()> {
        // Withdraw tokens, remove stake
        ctx.accounts.withdraw_tokens(amount)
    }

    // Execute a proposal
    pub fn execute_proposal(ctx: Context<FinalizeProposal>) -> Result<()> {
        // Pay a proposal fee to DAO treasury
        ctx.accounts.execute_proposal()
    }

    // Cleanup a failed proposal
    pub fn cleanup_proposal(ctx: Context<FinalizeProposal>) -> Result<()> {
        // Pay a proposal fee to DAO treasury
        ctx.accounts.cleanup_proposal()
    }

    
    // issue goverance token
    pub fn issue_tokens(ctx: Context<IssueTokens>) -> Result<()> {
        ctx.accounts.deposit_sol()?;
        ctx.accounts.issue_tokens()
    }

    // Close a voting position in an active proposal
    pub fn remove_vote(ctx: Context<RemoveOrCleanupVote>) -> Result<()> {
        // Decrement votes for user and proposal
        ctx.accounts.remove_vote()
    }

    // Close a voting position after a proposal has passed/expired
    pub fn cleanup_vote(ctx: Context<RemoveOrCleanupVote>) -> Result<()> {
        // Decrement votes for user
        ctx.accounts.cleanup_vote()
    }

    pub fn get_member_state(ctx: Context<GetMemberState>) -> Result<MemberStateView> {
        ctx.accounts.get_member_state()
    }
}
