use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer as transfer_spl, Mint, Token, TokenAccount, Transfer as TransferSpl},
};

use crate::state::{setup::DaoSetup, MemberState, StakeState};

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    owner: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = owner
    )]
    owner_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"vault", config.key().as_ref(), owner.key().as_ref()],
        bump = stake_state.vault_bump,
        token::mint = mint,
        token::authority = auth
    )]
    stake_ata: Account<'info, TokenAccount>,
    #[account(
        seeds=[b"auth", config.key().as_ref(), owner.key().as_ref()],
        bump = stake_state.auth_bump
    )]
    ///CHECK: This is safe. It's just used to sign things
    auth: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds=[b"mint", config.key().as_ref()],
        bump = config.mint_bump
    )]
    mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds=[b"stake", config.seed.to_le_bytes().as_ref()],
        bump = stake_state.state_bump
    )]
    stake_state: Account<'info, StakeState>,
    #[account(
        seeds=[b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    config: Account<'info, DaoSetup>,
    #[account(
        init_if_needed,
        payer = owner,
        space = MemberState::LEN,
        seeds = [b"member", config.key().as_ref(), owner.key().as_ref()],
        bump
    )]
    member_state: Account<'info, MemberState>,
    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
}

impl<'info> Stake<'info> {
    pub fn deposit_tokens(
        &mut self,
        amount: u64,
        member_state_bump: u8,
    ) -> Result<()> {
        // Stake tokens
        self.stake_state.stake(amount)?;

        // Transfer tokens
        let accounts = TransferSpl {
            from: self.owner_ata.to_account_info(),
            to: self.stake_ata.to_account_info(),
            authority: self.owner.to_account_info()
        };
        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            accounts
        );
        transfer_spl(ctx, amount)?;

        // Initialize member state if it's a new account
        if self.member_state.join_date == 0 {
            self.member_state.init(
                self.owner.key(),
                member_state_bump,
            )?;
        }

        Ok(())

    } 

    pub fn withdraw_tokens(&mut self, amount: u64) -> Result<()> {
        self.stake_state.unstake(amount)?;

        let accounts = TransferSpl {
            from: self.stake_ata.to_account_info(),
            to: self.owner_ata.to_account_info(),
            authority: self.auth.to_account_info(),
        };

        let seeds = &[
            &b"auth"[..],
            &self.config.key().to_bytes()[..],
            &self.auth.key().to_bytes()[..],
            &[self.stake_state.auth_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds,
        );

        transfer_spl(ctx, amount)
    }
}
