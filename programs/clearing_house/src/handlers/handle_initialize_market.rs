use crate::errors::Errors;
use crate::math::amm;
use crate::math::bn::U192;
use crate::{margin_validation, math_error, state::*};
use anchor_lang::prelude::*;

#[inline(always)]
pub fn handle_initialize_market(
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
    let markets = &mut ctx.accounts.markets.load_mut()?;
    let market = markets.get_market(market_index);
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;
    let clock_slot = clock.slot;
    if market.is_initialized() {
        return err!(Errors::MarketIndexAlreadyInitialized);
    }

    // 要求初始状态时 base/quote 储备量相等（1:1 peg），确保初始价格公允
    if amm_base_asset_reserve != amm_quote_asset_reserve {
        return err!(Errors::InvalidInitialPeg);
    }

    // 基于储备量和 peg 乘数计算初始价格
    // 由于amm_base_asset_reserve与amm_base_asset_reserve为1:1，所以只能通过设定amm_peg_multiplier值来调整初始价格
    let init_mark_price = amm::calculate_price(
        amm_base_asset_reserve,
        amm_base_asset_reserve,
        amm_peg_multiplier,
    )?;

    // AMM恒积公式校验，检查此时A*B不溢出
    _ = U192::from(amm_base_asset_reserve)
        .checked_mul(U192::from(amm_quote_asset_reserve))
        .ok_or_else(math_error!())?;

    // 根据flag oracle_source，从oracle中读取价格（已调整为市场标记价格精度）
    let OraclePriceData {
        price: oracle_price,
        ..
    } = match oracle_source {
        OracleSource::Pyth => market
            .amm
            .get_pyth_price(&ctx.accounts.oracle, clock_slot)
            .unwrap(),
        // todo: swtich board
        OracleSource::Switchboard => todo!(),
    };

    // todo: get twap price
    let last_oracle_twap_price = match oracle_source {
        // todo: add pyth twap update in context
        OracleSource::Pyth => market.amm.get_pyth_ema_price(&ctx.accounts.oracle)?,
        OracleSource::Switchboard => todo!(),
    };

    // 检验初始保证金率、部分平仓保证金率和维持保证金率
    margin_validation::margin_validation(
        margin_ratio_initial,
        margin_ratio_partial,
        margin_ratio_maintenance,
    )?;

    markets.markets[Markets::index_from_u64(market_index)] = Market {
        base_asset_amount_long: 0,
        base_asset_amount_short: 0,
        base_asset_amount: 0,
        open_interest: 0,
        amm: AMM {
            base_asset_reserve: amm_base_asset_reserve,
            quote_asset_reserve: amm_quote_asset_reserve,
            sqrt_k: amm_base_asset_reserve,
            cumulative_repeg_rebate_long: 0,
            cumulative_repeg_rebate_short: 0,
            cumulative_funding_rate_long: 0,
            cumulative_funding_rate_short: 0,
            last_funding_rate: 0,
            last_funding_rate_ts: now,
            funding_period: amm_periodicity,
            peg_multiplier: amm_peg_multiplier,
            total_fee: 0,
            total_fee_minus_distributions: 0,
            total_fee_withdrawn: 0,
            minimum_base_asset_trade_size: 10000000,
            mininum_quote_asset_trade_size: 10000000,
            last_mark_price_twap: init_mark_price,
            last_mark_price_twap_ts: now,
            last_oracle_price_twap_ts: now,
            last_oracle_price_twap: last_oracle_twap_price,
            oracle: *ctx.accounts.oracle.key,
            last_oracle_price: oracle_price,
            base_spread: 0,
            oracle_source,
            padding: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        },
        margin_ratio_initial,
        margin_ratio_partial,
        margin_ratio_maintenance,
        initialized: 1,
        padding0: [0, 0, 0],
        padding1: [0, 0, 0, 0],
    };

    Ok(())
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
