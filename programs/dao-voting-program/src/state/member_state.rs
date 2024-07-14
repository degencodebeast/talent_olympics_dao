use crate::{constants::*, errors::DaoError};
use anchor_lang::prelude::*;

#[account]
pub struct MemberState {
    pub address: Pubkey,
    pub reward_points: u64,
    pub total_votes_cast: u64,
    pub proposals_created: u64,
    pub successful_proposals: u64,
    pub join_date: i64,
    pub reputation_score: u64,
    pub base_voting_points: u64,
    pub bonus_voting_points: u64,
    pub proposal_creation_points: u64,
    pub proposal_success_points: u64,
    pub forfeited_points: u64,
    pub bump: u8,
}

impl MemberState {
    pub const LEN: usize = PUBKEY_LENGTH +    // address: Pubkey
    U64_LENGTH +       // reward_points: u64
    U64_LENGTH +       // total_votes_cast: u64
    U64_LENGTH +       // proposals_created: u64
    U64_LENGTH +       // successful_proposals: u64
    U64_LENGTH +       // join_date: i64 (i64 has the same size as u64)
    U64_LENGTH +       // reputation_score: u64
    U64_LENGTH +       // base_voting_points: u64
    U64_LENGTH +       // bonus_voting_points: u64
    U64_LENGTH +       // proposal_creation_points: u64
    U64_LENGTH +       // proposal_success_points: u64
    U64_LENGTH +       // forfeited_points: u64
    U8_LENGTH +        // bump: u8
    8; // Discriminator (added by Anchor)

    pub fn init(&mut self, address: Pubkey, bump: u8) -> Result<()> {
        self.address = address;
        self.reward_points = 0;
        self.total_votes_cast = 0;
        self.proposals_created = 0;
        self.successful_proposals = 0;
        self.join_date = Clock::get()?.unix_timestamp;
        self.reputation_score = 0;
        self.base_voting_points = 0;
        self.bonus_voting_points = 0;
        self.proposal_creation_points = 0;
        self.proposal_success_points = 0;
        self.forfeited_points = 0;
        self.bump = bump;
        Ok(())
    }

    pub fn add_vote_points(&mut self, base_points: u64) -> Result<()> {
        self.base_voting_points = self
            .base_voting_points
            .checked_add(base_points)
            .ok_or(DaoError::Overflow)?;
        self.total_votes_cast = self
            .total_votes_cast
            .checked_add(1)
            .ok_or(DaoError::Overflow)?;
        self.update_reward_points()
    }

    pub fn add_vote_bonus(&mut self, bonus_points: u64) -> Result<()> {
        self.bonus_voting_points = self
            .bonus_voting_points
            .checked_add(bonus_points)
            .ok_or(DaoError::Overflow)?;
        self.update_reward_points()
    }

    pub fn add_proposal_points(&mut self, points: u64) -> Result<()> {
        self.proposal_creation_points = self
            .proposal_creation_points
            .checked_add(points)
            .ok_or(DaoError::Overflow)?;
        self.proposals_created = self
            .proposals_created
            .checked_add(1)
            .ok_or(DaoError::Overflow)?;
        self.update_reward_points()
    }

    pub fn add_proposal_success_points(&mut self, points: u64) -> Result<()> {
        self.proposal_success_points = self
            .proposal_success_points
            .checked_add(points)
            .ok_or(DaoError::Overflow)?;
        self.successful_proposals = self
            .successful_proposals
            .checked_add(1)
            .ok_or(DaoError::Overflow)?;
        self.update_reward_points()
    }

    pub fn update_reward_points(&mut self) -> Result<()> {
        self.reward_points = self
            .base_voting_points
            .checked_add(self.bonus_voting_points)
            .and_then(|sum| sum.checked_add(self.proposal_creation_points))
            .and_then(|sum| sum.checked_add(self.proposal_success_points))
            .ok_or(DaoError::Overflow)?;
        Ok(())
    }

    pub fn slash_vote_points(&mut self, points: u64) -> Result<()> {
        let old_points = self.base_voting_points;
        self.base_voting_points = self.base_voting_points.saturating_sub(points);
        let actual_deduction = old_points - self.base_voting_points;
        if actual_deduction < points {
            self.forfeited_points = self
                .forfeited_points
                .saturating_add(points - actual_deduction);
        }
        self.update_reward_points()
    }

    pub fn update_reputation(&mut self, change: i64) -> Result<()> {
        if change >= 0 {
            self.reputation_score = self.reputation_score.saturating_add(change as u64);
        } else {
            self.reputation_score = self.reputation_score.saturating_sub((-change) as u64);
        }
        Ok(())
    }

    pub fn get_member_state(&self) -> MemberStateView {
        MemberStateView {
            address: self.address,
            reward_points: self.reward_points,
            total_votes_cast: self.total_votes_cast,
            proposals_created: self.proposals_created,
            successful_proposals: self.successful_proposals,
            join_date: self.join_date,
            reputation_score: self.reputation_score,
            base_voting_points: self.base_voting_points,
            bonus_voting_points: self.bonus_voting_points,
            proposal_creation_points: self.proposal_creation_points,
            proposal_success_points: self.proposal_success_points,
            forfeited_points: self.forfeited_points,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MemberStateView {
    pub address: Pubkey,
    pub reward_points: u64,
    pub total_votes_cast: u64,
    pub proposals_created: u64,
    pub successful_proposals: u64,
    pub join_date: i64,
    pub reputation_score: u64,
    pub base_voting_points: u64,
    pub bonus_voting_points: u64,
    pub proposal_creation_points: u64,
    pub proposal_success_points: u64,
    pub forfeited_points: u64,
}
