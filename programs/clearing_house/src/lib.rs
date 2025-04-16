#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use handlers::*;
use state::*;

pub mod controller;
pub mod errors;
pub mod handlers;
pub mod margin_validation;
pub mod math;
pub mod optional_accounts;
pub mod state;

use controller::position::PositionDirection;

declare_id!("3LptehCCdJcnsG8DaFJKqCGLorUswXYmmkCTkrzTjh1D");

#[program]
pub mod clearing_house {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, admin_controls_prices: bool) -> Result<()> {
        handle_initialize(ctx, admin_controls_prices)
    }

    pub fn intialize_history(ctx: Context<InitializeHistory>) -> Result<()> {
        handle_initialize_history(ctx)
    }

    pub fn initialize_order_state(ctx: Context<InitializeOrderState>) -> Result<()> {
        handle_initialize_order_state(ctx)
    }

    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        market_index: u64,
        // AMM base资产（如 SOL）初始储备量，决定初始流动性深度
        amm_base_asset_reserve: u128,
        // AMM quote资产（如 USDC）初始储备量，需与base资产相等
        amm_quote_asset_reserve: u128,
        // AMM 的再平衡周期（秒），影响资金费率调整频率（即资金费率的计算周期）
        amm_periodicity: i64,
        // 锚定乘数，用于调整 AMM 价格与预言机价格的偏差
        amm_peg_multiplier: u128,
        // 预言机类型（Pyth/Switchboard）
        oracle_source: OracleSource,
        // 初始保证金率（如 2000 = 20%），控制开仓最低抵押率
        margin_ratio_initial: u32,
        // 部分平仓保证金率，触发部分清算的阈值
        margin_ratio_partial: u32,
        // 维持保证金率，触发全额清算的阈值
        margin_ratio_maintenance: u32,
    ) -> Result<()> {
        handle_initialize_market(
            ctx,
            market_index,
            amm_base_asset_reserve,
            amm_quote_asset_reserve,
            amm_periodicity,
            amm_peg_multiplier,
            oracle_source,
            margin_ratio_initial,
            margin_ratio_partial,
            margin_ratio_maintenance,
        )
    }

    pub fn initialize_user(
        ctx: Context<InitializeUser>,
        optional_accounts: handle_user_initialization::InitializeUserOptionalAccounts,
    ) -> Result<()> {
        handle_user_initialization::initialize(
            &ctx.accounts.state,
            &mut ctx.accounts.user,
            &ctx.accounts.user_postions,
            &ctx.accounts.signer,
            ctx.remaining_accounts,
            optional_accounts,
        )
    }

    pub fn initialize_user_with_explicit_payer(
        ctx: Context<InitializeUserWithExplicitPayer>,
        optional_accounts: handle_user_initialization::InitializeUserOptionalAccounts,
    ) -> Result<()> {
        handle_user_initialization::initialize(
            &ctx.accounts.state,
            &mut ctx.accounts.user,
            &ctx.accounts.user_postions,
            &ctx.accounts.authority,
            ctx.remaining_accounts,
            optional_accounts,
        )
    }

    pub fn deposit_collateral(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
        handle_deposit_collateral(ctx, amount)
    }
}
