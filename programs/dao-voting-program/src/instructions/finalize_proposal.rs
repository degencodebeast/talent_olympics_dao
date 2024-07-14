use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};
use crate::{state::{setup::DaoSetup, Proposal, ProposalType}, errors::DaoError};

// This struct defines the accounts required for the CleanupProposal instruction
#[derive(Accounts)]
pub struct FinalizeProposal<'info> {
    #[account(mut)]
    initializer: Signer<'info>,  // The user initiating the cleanup or execution

    #[account(mut)]
    payee: UncheckedAccount<'info>,  // The account that receives bounty payouts (if applicable)

    #[account(
        mut,
        close = treasury,  // The proposal account will be closed and its rent sent to the treasury
        seeds=[b"proposal", config.key().as_ref(), proposal.id.to_le_bytes().as_ref()],
        bump = proposal.bump
    )]
    proposal: Account<'info, Proposal>,  // The proposal account being cleaned up or executed

    #[account(
        seeds=[b"treasury", config.key().as_ref()],
        bump = config.treasury_bump
    )]
    treasury: SystemAccount<'info>,  // The DAO's treasury account

    #[account(
        seeds=[b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    config: Account<'info, DaoSetup>,  // The DAO's configuration account

    system_program: Program<'info, System>  // Required for SOL transfers
}

impl<'info> FinalizeProposal<'info> {
    // This function cleans up a failed proposal
    pub fn cleanup_proposal(
        &mut self
    ) -> Result<()> {
        let _ = self.proposal.try_finalize();  // Attempt to finalize the proposal
        self.proposal.is_failed()?;  // Ensure the proposal has failed
        Ok(())
    }

    // This function executes a successful proposal
    pub fn execute_proposal(
        &mut self
    ) -> Result<()> {
        self.proposal.try_finalize()?;  // Attempt to finalize the proposal
        self.proposal.is_succeeded()?;  // Ensure the proposal has succeeded
        match self.proposal.proposal {
            ProposalType::Bounty(payee, payout) => self.payout_bounty(payee, payout),
            ProposalType::Executable => self.execute_tx(),
            ProposalType::Vote => self.finalize_vote(),
        }
    }

    // This function finalizes a vote proposal by logging the results
    pub fn finalize_vote(&self) -> Result<()> {
        msg!("Vote result: {} / {}", self.proposal.votes, self.proposal.quorum);
        msg!("Vote has {:?}", self.proposal.result);
        Ok(())
    }

    // This function pays out a bounty to the specified payee
    pub fn payout_bounty(
        &self,
        payee: Pubkey,
        payout: u64
    ) -> Result<()> {
        require_keys_eq!(self.payee.key(), payee);  // Ensure the payee account matches the proposal
        let accounts = Transfer {
            from: self.treasury.to_account_info(),
            to: self.payee.to_account_info()
        };
        let seeds = &[
            &b"auth"[..],
            &self.config.key().to_bytes()[..],
            &[self.config.auth_bump],
        ];
        let signer_seeds = &[&seeds[..]];
        let ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            accounts,
            signer_seeds
        );
        transfer(ctx, payout)  // Transfer the bounty from the treasury to the payee
    }

    // This function is a placeholder for executing a transaction proposal
    pub fn execute_tx(
        &self
    ) -> Result<()> {
        // Placeholder for executing a transaction
        // Implementation would depend on the specific requirements
        Ok(())
    }
}