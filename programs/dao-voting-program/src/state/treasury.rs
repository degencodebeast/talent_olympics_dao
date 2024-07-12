#[account]
pub struct Treasury {
    pub balance: u64,
    pub total_rewards_distributed: u64,
    pub bump: u8,
}