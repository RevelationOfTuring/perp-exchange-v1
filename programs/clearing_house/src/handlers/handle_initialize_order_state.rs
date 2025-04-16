use crate::errors::Errors;
use crate::state::*;
use anchor_lang::prelude::*;
use std::mem::size_of;

#[inline(always)]
pub fn handle_initialize_order_state(ctx: Context<InitializeOrderState>) -> Result<()> {
    let state = &mut ctx.accounts.state.load_mut()?;
    // 判断state中是否已经初始化过order state的
    if !state.order_state.eq(&Pubkey::default()) {
        return err!(Errors::OrderStateAlreadyInitialized);
    }

    // 在state中设置order state的key
    state.order_state = ctx.accounts.order_state.key();
    // 初始化order history
    ctx.accounts.order_history.load_init()?;
    // 初始化order state
    **ctx.accounts.order_state = OrderState {
        order_history: ctx.accounts.order_history.key(),
        order_filler_reward_structure: OrderFillerRewardStructure {
            reward_numerator: 1,
            reward_denominator: 10,
            time_based_reward_lower_bound: 10_000, // 1 cent
        },
        min_order_quote_asset_amount: 500_000, // 50 cents
    };

    Ok(())
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
