use crate::state::{
    history::{
        curve_history::CurveHistory, deposit_history::DepositHistory,
        funding_payment_history::FundingPaymentHistory, funding_rate_history::FundingRateHistory,
        liquidation_history::LiquidationHistory, order_history::OrderHistory,
        trade_history::TradeHistory,
    },
    market::Markets,
    order_state::OrderState,
    state::State,
    user::{User, UserPositions},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use std::mem::size_of;

#[derive(Accounts)]
pub struct Initialize<'info> {
    // 该signer会成为State中的admin
    #[account(mut)]
    pub admin: Signer<'info>,
    // 1. 创建pda，用于存储State
    #[account(zero)]
    pub state: AccountLoader<'info, State>,
    // 2. 抵押品mint
    pub collateral_mint: Box<Account<'info, Mint>>,
    // 3. 创建抵押品vault（即owner为本program的一个ata，token种类为collateral_mint，authority为collateral_vault_authority）
    #[account(
        init,
        payer = admin,
        seeds = [b"collateral_vault".as_ref()],
        bump,
        token::mint = collateral_mint,
        token::authority = collateral_vault_authority
    )]
    pub collateral_vault: Box<Account<'info, TokenAccount>>,
    // 4. 抵押品vault的authority
    /// CHECK: checked in `initialize`
    pub collateral_vault_authority: UncheckedAccount<'info>,
    // 5. 创建保证金Vault（即owner为本program的一个ata，token种类为collateral_mint，authority为insurance_vault_authority）
    #[account(
        init,
        payer = admin,
        seeds = [b"insurance_vault".as_ref()],
        bump,
        token::mint = collateral_mint,
        token::authority = insurance_vault_authority
    )]
    pub insurance_vault: Box<Account<'info, TokenAccount>>,
    // 6. 保证金vault的authority
    /// CHECK: checked in `initialize`
    pub insurance_vault_authority: UncheckedAccount<'info>,
    // 7. markets账户
    #[account(zero)]
    pub markets: AccountLoader<'info, Markets>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeHistory<'info> {
    pub admin: Signer<'info>,
    #[account(
        mut,
        has_one = admin
    )]
    pub state: AccountLoader<'info, State>,
    #[account(zero)]
    pub funding_payment_history: AccountLoader<'info, FundingPaymentHistory>,
    #[account(zero)]
    pub trade_history: AccountLoader<'info, TradeHistory>,
    #[account(zero)]
    pub liquidation_history: AccountLoader<'info, LiquidationHistory>,
    #[account(zero)]
    pub deposit_history: AccountLoader<'info, DepositHistory>,
    #[account(zero)]
    pub funding_rate_history: AccountLoader<'info, FundingRateHistory>,
    #[account(zero)]
    pub curve_history: AccountLoader<'info, CurveHistory>,
}

#[derive(Accounts)]
pub struct InitializeOrderState<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        has_one = admin
    )]
    pub state: AccountLoader<'info, State>,
    #[account(
        init,
        payer = admin,
        space = 8 + size_of::<OrderState>(),
        seeds = [b"order_state".as_ref()],
        bump,
    )]
    pub order_state: Box<Account<'info, OrderState>>,
    #[account(zero)]
    pub order_history: AccountLoader<'info, OrderHistory>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    pub admin: Signer<'info>,
    #[account(
        has_one = admin
    )]
    pub state: AccountLoader<'info, State>,
    #[account(
        mut,
        constraint = state.load()?.markets.eq(&markets.key())
    )]
    pub markets: AccountLoader<'info, Markets>,
    /// CHECK: checked in `initialize_market`
    pub oracle: UncheckedAccount<'info>,
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

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub state: AccountLoader<'info, State>,
    #[account(
        init,
        payer = signer,
        space = 8 + size_of::<User>(),
        seeds = [b"user", signer.key.as_ref()],
        bump
    )]
    pub user: Box<Account<'info, User>>,
    #[account(
        init,
        payer = signer,
        space = 8 + size_of::<UserPositions>(),
    )]
    pub user_postions: AccountLoader<'info, UserPositions>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeUserWithExplicitPayer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    pub state: AccountLoader<'info, State>,
    #[account(
        init,
        payer = payer,
        space = 8 + size_of::<User>(),
        seeds = [b"user", authority.key.as_ref()],
        bump
    )]
    pub user: Box<Account<'info, User>>,
    #[account(
        init,
        payer = payer,
        space = 8 + size_of::<UserPositions>(),
    )]
    pub user_postions: AccountLoader<'info, UserPositions>,
    pub system_program: Program<'info, System>,
}
