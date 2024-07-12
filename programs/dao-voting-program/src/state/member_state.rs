#[account]
pub struct MemberState {
    pub address: Pubkey,
    pub reward_points: u64,
    pub total_votes_cast: u64,
    pub join_date: i64,
    pub reputation_score: u64,
    pub bump: u8,
}