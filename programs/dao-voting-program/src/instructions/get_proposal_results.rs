use anchor_lang::prelude::*;
use crate::state::{setup::DaoSetup, Proposal, ProposalResults};
//use crate::errors::DaoError;

#[derive(Accounts)]
pub struct GetProposalResults<'info> {
    // The user requesting the results (doesn't need to be mutable)
    pub user: Signer<'info>,

    #[account(
        seeds=[b"proposal", config.key().as_ref(), proposal.id.to_le_bytes().as_ref()],
        bump = proposal.bump,
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        seeds=[b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    pub config: Account<'info, DaoSetup>,
}

impl<'info> GetProposalResults<'info> {
    pub fn get_results(&self) -> Result<ProposalResults> {
        // Check if the proposal has expired and finalize if necessary
        self.check_and_finalize()?;

        // Get and return the results
        Ok(self.proposal.get_results())
    }

    fn check_and_finalize(&self) -> Result<()> {
        let binding = self.proposal.to_account_info();
        let mut proposal = binding.try_borrow_mut_data()?;
        let mut proposal_data = Proposal::try_deserialize(&mut &proposal[..])?;

        // Check if the proposal has expired
        if Clock::get()?.slot >= proposal_data.expiry {
            proposal_data.try_finalize()?;
            proposal_data.serialize(&mut &mut proposal[..])?;
        }

        Ok(())
    }
}
