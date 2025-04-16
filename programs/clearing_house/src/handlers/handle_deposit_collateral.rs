use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

pub fn handle_deposit_collateral(ctx: Context<DepositCollateral>, _amount: u64) -> Result<()> {
    let user = &mut ctx.accounts.user;
    let _now = Clock::get()?.unix_timestamp;

    let _collateral_before = user.collateral;
    let _cumculative_deposits_before = user.cumculative_deposits;

    let _markets = ctx.accounts.markets.load()?;
    let _user_position = ctx.accounts.user_positions.load_mut()?;

    Ok(())
}

#[derive(Accounts)]
pub struct DepositCollateral<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub state: AccountLoader<'info, State>,
    #[account(
        mut,
        has_one = authority,
        // 保证user与user_positions的一致性
        constraint = user.positons.key().eq(&user_positions.key())
    )]
    pub user: Box<Account<'info, User>>,
    #[account(
        mut,
        constraint = state.load()?.collateral_vault.eq(&collateral_vault.key())
    )]
    pub collateral_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub user_collateral_account: Box<Account<'info, TokenAccount>>,
    #[account(
        constraint = state.load()?.markets.eq(&markets.key())
    )]
    pub markets: AccountLoader<'info, Markets>,
    #[account(
        mut,
        // 保证user与user_positions的一致性
        has_one = user       
    )]
    pub user_positions: AccountLoader<'info, UserPositions>,
    #[account(
        mut,
        constraint = state.load()?.funding_payment_history.eq(&funding_payment_history.key())
    )]
    pub funding_payment_history: AccountLoader<'info, FundingPaymentHistory>,
    #[account(
        mut,
        constraint = state.load()?.deposit_history.eq(&deposit_history.key())
    )]
    pub deposit_history: AccountLoader<'info, DepositHistory>,
    pub token_program: Program<'info, Token>,
}
