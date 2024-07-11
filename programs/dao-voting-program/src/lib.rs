use anchor_lang::prelude::*;

mod instructions;
use instructions::*;
mod state;
mod errors;
mod constants;

declare_id!("HScCa2Qkqn5DRFDXySiwhtTMNJs87hdo8DjsBxvBfyn6");

#[program]
pub mod dao_voting_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}