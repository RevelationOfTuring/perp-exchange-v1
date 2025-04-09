#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use context::*;
use errors::Errors;
use math::{bn::U192, constant::*};
use state::state::*;
use state::{market::*, order_state::*};

pub mod context;
pub mod controller;
pub mod errors;
pub mod margin_validation;
pub mod math;
pub mod optional_accounts;
pub mod state;
pub mod user_initialization;

use controller::position::PositionDirection;

declare_id!("DmYiroqr7gNKTANvTmVLbVbpFQWAganFVBhFioKSc5Da");

#[program]
pub mod clearing_house {
    use crate::math::amm;

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, admin_controls_prices: bool) -> Result<()> {
        // collateral_vault账户地址（pda，seeds为[b"collateral_vault"]）
        let collateral_vault_key = ctx.accounts.collateral_vault.to_account_info().key;
        // 生成pda地址（作为collateral_vault的authority）和对应bump
        // seeds为[collateral_account地址]
        let (collateral_vault_authority, collateral_vault_authority_bump) =
            Pubkey::find_program_address(&[collateral_vault_key.as_ref()], ctx.program_id);
        require_keys_eq!(
            ctx.accounts.collateral_vault_authority.key(),
            collateral_vault_authority,
            Errors::InvalidCollateralVaultAuthority
        );

        // insurance_vault账户地址（pda，seeds为[b"insurance_vault"]）
        let insurance_vault_key = ctx.accounts.insurance_vault.to_account_info().key;
        // 生成pda地址（作为insurance_vault的authority）和对应bump
        // seeds为[insurance_vault账户地址]
        let (insurance_vault_authority, insurance_vault_authority_bump) =
            Pubkey::find_program_address(&[insurance_vault_key.as_ref()], ctx.program_id);
        require_keys_eq!(
            ctx.accounts.insurance_vault_authority.key(),
            insurance_vault_authority,
            Errors::InvalidInsuranceVaultAuthority
        );

        ctx.accounts.markets.load_init()?;

        let state = &mut ctx.accounts.state.load_init()?;
        let default_pubkey = Pubkey::default();
        **state = State {
            exchange_paused: 0,
            funding_paused: 0,
            admin_controls_prices: if admin_controls_prices { 1 } else { 0 },
            collateral_vault_authority_nonce: collateral_vault_authority_bump,
            insurance_vault_authority_nonce: insurance_vault_authority_bump,
            padding0: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],

            admin: *ctx.accounts.admin.key,
            collateral_mint: ctx.accounts.collateral_mint.key(),
            collateral_vault: *collateral_vault_key,
            collateral_vault_authority,
            // deposit_history/trade_history/funding_rate_history/funding_payment_history/liquidation_history/curve_history
            // 这六个history会被设置为Pubkey的默认值，进一步的设置会在initialize_history中做
            deposit_history: default_pubkey,
            trade_history: default_pubkey,
            funding_payment_history: default_pubkey,
            funding_rate_history: default_pubkey,
            liquidation_history: default_pubkey,
            curve_history: default_pubkey,
            insurance_vault: *insurance_vault_key,
            insurance_vault_authority,
            markets: *ctx.accounts.markets.to_account_info().key,
            // 20%
            margin_ratio_initial: 2000,
            margin_ratio_maintenance: 625,
            margin_ratio_partial: 500,
            partial_liquidation_close_percentage_numerator: 25,
            partial_liquidation_close_percentage_denominator: 100,
            partial_liquidation_penalty_percentage_numberator: 25,
            partial_liquidation_penalty_percentage_denominator: 1000,
            full_liquidation_penalty_percentage_numerator: 1,
            full_liquidation_penalty_percentage_denominator: 1,
            partial_liquidation_liquidator_share_denominator: 2,
            full_liquidation_liquidator_share_denominator: 20,
            fee_structure: FeeStructure {
                fee_numerator: DEFAULT_FEE_NUMERATOR,
                fee_denominator: DEFAULT_FEE_DENOMINATOR,
                discount_token_tiers: DiscountTokenTiers {
                    first_tier: DiscountTokenTier {
                        minimun_balance: DEFAULT_DISCOUNT_TOKEN_FIRST_TIER_MINIMUM_BALANCE,
                        discount_numerator: DEFAULT_DISCOUNT_TOKEN_FIRST_TIER_DISCOUNT_NUMERATOR,
                        discount_denominator:
                            DEFAULT_DISCOUNT_TOKEN_FIRST_TIER_DISCOUNT_DENOMINATOR,
                        padding: [0, 0, 0, 0, 0, 0, 0, 0],
                    },
                    second_tier: DiscountTokenTier {
                        minimun_balance: DEFAULT_DISCOUNT_TOKEN_SECOND_TIER_MINIMUM_BALANCE,
                        discount_numerator: DEFAULT_DISCOUNT_TOKEN_SECOND_TIER_DISCOUNT_NUMERATOR,
                        discount_denominator:
                            DEFAULT_DISCOUNT_TOKEN_SECOND_TIER_DISCOUNT_DENOMINATOR,
                        padding: [0, 0, 0, 0, 0, 0, 0, 0],
                    },
                    third_tier: DiscountTokenTier {
                        minimun_balance: DEFAULT_DISCOUNT_TOKEN_THIRD_TIER_MINIMUM_BALANCE,
                        discount_numerator: DEFAULT_DISCOUNT_TOKEN_THIRD_TIER_DISCOUNT_NUMERATOR,
                        discount_denominator:
                            DEFAULT_DISCOUNT_TOKEN_THIRD_TIER_DISCOUNT_DENOMINATOR,
                        padding: [0, 0, 0, 0, 0, 0, 0, 0],
                    },
                    fourth_tier: DiscountTokenTier {
                        minimun_balance: DEFAULT_DISCOUNT_TOKEN_FOURTH_TIER_MINIMUM_BALANCE,
                        discount_numerator: DEFAULT_DISCOUNT_TOKEN_FOURTH_TIER_DISCOUNT_NUMERATOR,
                        discount_denominator:
                            DEFAULT_DISCOUNT_TOKEN_FOURTH_TIER_DISCOUNT_DENOMINATOR,
                        padding: [0, 0, 0, 0, 0, 0, 0, 0],
                    },
                },
                referral_discount: ReferralDiscount {
                    referral_reward_numerator: DEFAULT_REFERRER_REWARD_NUMERATOR,
                    referral_reward_denominator: DEFAULT_REFERRER_REWARD_DENOMINATOR,
                    referee_discount_numerator: DEFAULT_REFEREE_DISCOUNT_NUMERATOR,
                    referee_discount_denominator: DEFAULT_REFEREE_DISCOUNT_DENOMINATOR,
                },
            },
            whitelist_mint: default_pubkey,
            discount_mint: default_pubkey,
            oracle_guard_rails: OracleGuardRails {
                price_divergence: PriceDivergenceGuardRails {
                    mark_oracle_divergence_numerator: 1,
                    mark_oracle_divergence_denominator: 10,
                },
                validity: ValidityGuardRails {
                    slots_before_stable: 1000,
                    confidence_interval_max_size: 4,
                    too_volatile_ratio: 5,
                    padding: [0, 0, 0, 0, 0, 0, 0, 0],
                },
                use_for_liquidations: 1,
                padding: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            },
            max_deposit: 0,
            extended_curve_history: default_pubkey,
            order_state: default_pubkey,
            padding1: [0, 0, 0, 0],
        };

        Ok(())
    }

    pub fn intialize_history(ctx: Context<InitializeHistory>) -> Result<()> {
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

    pub fn initialize_order_state(ctx: Context<InitializeOrderState>) -> Result<()> {
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
            OracleSource::SwitchBoard => todo!(),
        };

        // todo: get twap price
        let last_oracle_twap_price = match oracle_source {
            // todo: add pyth twap update in context
            OracleSource::Pyth => market.amm.get_pyth_ema_price(&ctx.accounts.oracle)?,
            OracleSource::SwitchBoard => todo!(),
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

    pub fn initialize_user(
        ctx: Context<InitializeUser>,
        optional_accounts: user_initialization::InitializeUserOptionalAccounts,
    ) -> Result<()> {
        user_initialization::initialize(
            &ctx.accounts.state,
            &mut ctx.accounts.user,
            &ctx.accounts.user_postions,
            &ctx.accounts.signer,
            ctx.remaining_accounts,
            optional_accounts,
        )
    }
}
