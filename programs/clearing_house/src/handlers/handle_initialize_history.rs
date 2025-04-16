use crate::errors::Errors;
use crate::state::*;
use anchor_lang::prelude::*;

#[inline(always)]
pub fn handle_initialize_history(ctx: Context<InitializeHistory>) -> Result<()> {
    let state = &mut ctx.accounts.state.load_mut()?;
    let default_pubkey = Pubkey::default();
    // 如果state中这6个Pubkey都不是Pubkey默认值时，就会报错（表明history已经初始化过）
    if !state.trade_history.eq(&default_pubkey)
        && !state.deposit_history.eq(&default_pubkey)
        && !state.liquidation_history.eq(&default_pubkey)
        && !state.funding_rate_history.eq(&default_pubkey)
        && !state.funding_payment_history.eq(&default_pubkey)
        && !state.curve_history.eq(&default_pubkey)
    {
        return err!(Errors::HistoriesAllInitialized);
    }

    // 初始化这6个history账户的data
    ctx.accounts.trade_history.load_init()?;
    ctx.accounts.deposit_history.load_init()?;
    ctx.accounts.liquidation_history.load_init()?;
    ctx.accounts.funding_rate_history.load_init()?;
    ctx.accounts.funding_payment_history.load_init()?;
    ctx.accounts.curve_history.load_init()?;

    state.trade_history = *ctx.accounts.trade_history.to_account_info().key;
    state.deposit_history = *ctx.accounts.deposit_history.to_account_info().key;
    state.liquidation_history = *ctx.accounts.liquidation_history.to_account_info().key;
    state.funding_rate_history = *ctx.accounts.funding_rate_history.to_account_info().key;
    state.funding_payment_history = *ctx.accounts.funding_payment_history.to_account_info().key;
    state.curve_history = *ctx.accounts.curve_history.to_account_info().key;

    Ok(())
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
