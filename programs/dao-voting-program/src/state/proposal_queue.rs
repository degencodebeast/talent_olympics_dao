#[account]
pub struct ProposalQueue {
    pub active_proposals: Vec<u64>,
    pub completed_proposals: Vec<u64>,
    pub bump: u8,
}