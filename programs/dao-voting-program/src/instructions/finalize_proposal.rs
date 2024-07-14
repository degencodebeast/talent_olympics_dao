use crate::{
    constants::*,
    errors::DaoError,
    state::{setup::DaoSetup, MemberState, Proposal, ProposalType},
};
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

// This struct defines the accounts required for the CleanupProposal instruction
#[derive(Accounts)]
pub struct FinalizeProposal<'info> {
    #[account(mut)]
    initializer: Signer<'info>, // The user initiating the cleanup or execution

    /// CHECK: This account is not dangerous because we verify it matches the payee specified in the proposal
    #[account(mut)]
    payee: UncheckedAccount<'info>,

    #[account(
        mut,
        close = treasury,  // The proposal account will be closed and its rent sent to the treasury
        seeds=[b"proposal", config.key().as_ref(), proposal.id.to_le_bytes().as_ref()],
        bump = proposal.bump
    )]
    proposal: Account<'info, Proposal>, // The proposal account being cleaned up or executed

    #[account(
        mut,
        seeds=[b"member", config.key().as_ref(), proposal.proposer.key().as_ref()],
        bump = proposer_state.bump,
    )]
    proposer_state: Account<'info, MemberState>, //The state of the proposer account

    #[account(
        seeds=[b"treasury", config.key().as_ref()],
        bump = config.treasury_bump
    )]
    treasury: SystemAccount<'info>, // The DAO's treasury account

    #[account(
        seeds=[b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    config: Account<'info, DaoSetup>, // The DAO's configuration account

    system_program: Program<'info, System>, // Required for SOL transfers
}

impl<'info> FinalizeProposal<'info> {
    // This function cleans up a failed proposal
    pub fn cleanup_proposal(&mut self) -> Result<()> {
        let _ = self.proposal.try_finalize(); // Attempt to finalize the proposal
        self.proposal.is_failed()?; // Ensure the proposal has failed
        Ok(())
    }

    // This function executes a successful proposal
    pub fn execute_proposal(&mut self) -> Result<()> {
        self.proposal.try_finalize()?; // Attempt to finalize the proposal
        self.proposal.is_succeeded()?; // Ensure the proposal has succeeded

        // Add reward points and increase reputation for the proposer
        self.proposer_state
            .add_proposal_success_points(PROPOSAL_SUCCESS_POINTS)?;
        self.proposer_state
            .update_reputation(PROPOSAL_SUCCESS_REPUTATION_INCREASE)?;

        match self.proposal.proposal {
            ProposalType::Bounty(payee, payout) => self.payout_bounty(payee, payout),
            ProposalType::Executable => self.execute_tx(),
            ProposalType::Vote => self.finalize_vote(),
        }
    }

    // This function finalizes a vote proposal by logging the results
    pub fn finalize_vote(&self) -> Result<()> {
        msg!(
            "Vote result: {} / {}",
            self.proposal.votes,
            self.proposal.quorum
        );
        msg!("Vote has {:?}", self.proposal.result);
        Ok(())
    }

    // This function pays out a bounty to the specified payee
    pub fn payout_bounty(&self, payee: Pubkey, payout: u64) -> Result<()> {
        require_keys_eq!(self.payee.key(), payee); // Ensure the payee account matches the proposal
        let accounts = Transfer {
            from: self.treasury.to_account_info(),
            to: self.payee.to_account_info(),
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
            signer_seeds,
        );
        transfer(ctx, payout) // Transfer the bounty from the treasury to the payee
    }

    // This function is a placeholder for executing a transaction proposal
    pub fn execute_tx(&self) -> Result<()> {
        // Placeholder for executing a transaction
        // Implementation would depend on the specific requirements
        Ok(())
    }
}
