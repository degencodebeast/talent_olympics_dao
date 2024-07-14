use anchor_lang::prelude::*;
use crate::{
    state::{setup::DaoSetup, Proposal, StakeState, VoteState, MemberState, VoteType},
    errors::DaoError,
    constants::*,
};

#[derive(Accounts)]
pub struct Vote<'info> {
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
        init,
        payer = owner,
        seeds=[b"vote", proposal.key().as_ref(), owner.key().as_ref()],
        bump,
        space = VoteState::LEN
    )]
    vote: Account<'info, VoteState>,
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
    system_program: Program<'info, System>
}

impl<'info> Vote<'info> {
    pub fn vote(
        &mut self,
        amount: u64,
        vote_type: VoteType,
        bump: u8
    ) -> Result<()> {
        // Check if proposal is open
        self.proposal.is_open()?;
        // Check proposal hasn't expired
        self.proposal.check_expiry()?;
        // Ensure vote amount > 0
        require!(amount > 0, DaoError::InvalidVoteAmount);
        // Add vote to proposal
        self.proposal.add_vote(amount, vote_type)?;
        // Make sure user has staked
        self.stake_state.check_stake_amount(amount)?;
        // Add a vote account to the stake state
        self.stake_state.add_account()?;
        // Initialize vote
        self.vote.init(
            self.owner.key(),
            amount,
            vote_type,
            bump
        )?;

        // Award base voting points
        self.member_state.add_vote_points(BASE_VOTE_POINTS)?;

        // Increase reputation for voting
        self.member_state.update_reputation(VOTE_REPUTATION_INCREASE)?;

        Ok(())
    }
}