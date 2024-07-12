#[account]
pub struct GovernanceToken {
    pub mint: Pubkey,
    pub total_supply: u64,
    pub decimals: u8,
    pub bump: u8,
}