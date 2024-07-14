use crate::{
    constants::{BASE_VOTE_POINTS, VOTE_REPUTATION_DECREASE}, errors::DaoError, state::{setup::DaoSetup, MemberState, Proposal, StakeState, VoteState, VoteType}
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RemoveOrCleanupVote<'info> {
    #[account(mut)]
    owner: Signer<'info>,

    #[account(
        mut,
        seeds=[b"stake", config.key().as_ref(), owner.key().as_ref()],
        bump = stake_state.state_bump
    )]
    stake_state: Account<'info, StakeState>,

    #[account(
        mut,
        seeds=[b"proposal", config.key().as_ref(), proposal.id.to_le_bytes().as_ref()],
        bump = proposal.bump,
    )]
    proposal: Account<'info, Proposal>,

    #[account(
        mut,
        close = treasury,
        seeds=[b"vote", proposal.key().as_ref(), owner.key().as_ref()],
        bump = vote.bump
    )]
    vote: Account<'info, VoteState>,

    #[account(
        seeds=[b"treasury", config.key().as_ref()],
        bump = config.treasury_bump
    )]
    treasury: SystemAccount<'info>,

    #[account(
        mut,
        seeds=[b"member", config.key().as_ref(), owner.key().as_ref()],
        bump = member_state.bump,
    )]
    member_state: Account<'info, MemberState>,

    #[account(
        seeds=[b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    config: Account<'info, DaoSetup>,

    system_program: Program<'info, System>,
}

impl<'info> RemoveOrCleanupVote<'info> {
    pub fn cleanup_vote(&mut self) -> Result<()> {
        if self.proposal.is_open().is_ok() && self.proposal.check_expiry().is_ok() {
            return err!(DaoError::InvalidProposalStatus);
        }
        // Remove a vote account from the stake state
        self.stake_state.remove_account()
    }

    pub fn remove_vote(&mut self) -> Result<()> {
        // Check if the proposal is still open and not expired
        self.proposal.is_open()?;
        self.proposal.check_expiry()?;

        // Remove the vote from the proposal
        self.proposal
            .remove_vote(self.vote.amount, self.vote.vote_type)?;

        // Remove the vote account from the stake state
        self.stake_state.remove_account()?;

        // Slash reward points
        self.member_state.slash_vote_points(BASE_VOTE_POINTS)?;

        // Decrease reputation for removing vote
        self.member_state
            .update_reputation(VOTE_REPUTATION_DECREASE)?;
        
        Ok(())
    }
}
