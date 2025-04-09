#![allow(unexpected_cfgs)]
use anchor_lang::__private::bytemuck;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};
use std::mem::size_of;

declare_id!("EquvT3T5QAnj8t9sL9sCN3mXXzdhrugt3Esv3Bqjyemi");

#[program]
pub mod mock_usdc_faucet {
    use super::*;

    pub fn initialize_mock_usdc_faucet(ctx: Context<InitializeMockUSDCFaucet>) -> Result<()> {
        let mock_usdc_mint = ctx.accounts.usdc_mint.key();
        let seeds = &[&mock_usdc_mint.to_bytes()[..]];

        let (mint_authority_pda, mint_authority_pda_bump) =
            Pubkey::find_program_address(seeds, ctx.program_id);

        if ctx.accounts.usdc_mint.mint_authority.unwrap() != mint_authority_pda {
            return err!(Errors::UnmatchedMintAuthority);
        }

        **ctx.accounts.state = State {
            mint_authority_pda: mint_authority_pda,
            mint: mock_usdc_mint,
            mint_authority_pda_bump: mint_authority_pda_bump,
        };

        Ok(())
    }

    pub fn mint_to_user(ctx: Context<MintToUser>, amount: u64) -> Result<()> {
        let state = &**ctx.accounts.state;
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mock_usdc_mint.to_account_info(),
            to: ctx.accounts.reciever.to_account_info(),
            authority: ctx.accounts.mint_authority_pda.to_account_info(),
        };

        let pda_seeds = [
            &state.mint.to_bytes()[..],
            bytemuck::bytes_of(&state.mint_authority_pda_bump),
        ];
        let signer_seeds = &[&pda_seeds[..]];
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );

        token::mint_to(cpi_context, amount)
    }
}

#[derive(Accounts)]
pub struct MintToUser<'info> {
    #[account(mut)]
    pub mock_usdc_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub reciever: Box<Account<'info, TokenAccount>>,
    pub state: Box<Account<'info, State>>,
    /// CHECK: Checked by spl_token
    pub mint_authority_pda: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct InitializeMockUSDCFaucet<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = 8 + size_of::<State>(),
        seeds = [b"state".as_ref()],
        bump
    )]
    pub state: Box<Account<'info, State>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct State {
    mint: Pubkey,
    mint_authority_pda: Pubkey,
    mint_authority_pda_bump: u8,
}

#[error_code]
pub enum Errors {
    #[msg("unmatched mint authority")]
    UnmatchedMintAuthority,
}
