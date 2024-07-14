use anchor_lang::prelude::*;
use crate::state::{setup::DaoSetup, MemberState, MemberStateView};

#[derive(Accounts)]
pub struct GetMemberState<'info> {
    /// CHECK: This account is not written to, only used as a key for PDA derivation.
    pub member: AccountInfo<'info>,
    #[account(
        seeds=[b"member", config.key().as_ref(), member.key().as_ref()],
        bump = member_state.bump,
    )]
    pub member_state: Account<'info, MemberState>,
    #[account(
        seeds=[b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    pub config: Account<'info, DaoSetup>,
}

impl<'info> GetMemberState<'info> {
    pub fn get_member_state(&self) -> Result<MemberStateView> {
       
        // Get member details
        Ok(self.member_state.get_member_state())
    }
}



