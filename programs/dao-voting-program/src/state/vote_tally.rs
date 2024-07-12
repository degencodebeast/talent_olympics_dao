#[account]
pub struct VoteTally {
    pub proposal_id: u64,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub abstain_votes: u64,
    pub bump: u8,
}