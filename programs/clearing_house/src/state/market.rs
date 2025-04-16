use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;
use static_assertions::const_assert_eq;
use std::mem::size_of;

use crate::{
    errors::Errors,
    math::{
        amm,
        bn::ClearingHouseResult,
        cast::{cast, cast_to_i128, cast_to_i64, cast_to_u128},
        constant::MARK_PRICE_PRECISION,
    },
    math_error,
};

#[account(zero_copy)]
// markets账户里面存有最多64个Market的信息
pub struct Markets {
    pub markets: [Market; 64],
}

const_assert_eq!(size_of::<Markets>(), 31744);

impl Markets {
    // 将u64转换成usize，作为Markets.markets的索引
    pub fn index_from_u64(index: u64) -> usize {
        std::convert::TryInto::try_into(index).unwrap()
    }

    // 获得index对应的Market的不可变引用
    pub fn get_market(&self, index: u64) -> &Market {
        &self.markets[Self::index_from_u64(index)]
    }

    // 获得index对应的Market的可变引用
    pub fn get_market_mut(&mut self, index: u64) -> &mut Market {
        &mut self.markets[Self::index_from_u64(index)]
    }
}

#[zero_copy]
pub struct Market {
    pub base_asset_amount_long: i128, // 多头头寸的基础资产数量（正数表示）
    pub base_asset_amount_short: i128, // 空头头寸的基础资产数量（负数表示）
    pub base_asset_amount: i128, // 净市场偏差，即多头和空头头寸的净额（base_asset_amount_long + base_asset_amount_short）
    pub open_interest: u128,     // 持仓用户数量，表示当前有多少用户在市场中持有头寸
    pub amm: AMM,
    pub margin_ratio_initial: u32, // 初始保证金比例（开仓时要求的保证金比例）
    pub margin_ratio_partial: u32, // 部分清算保证金比例（当保证金低于此比例时可能触发部分清算）
    pub margin_ratio_maintenance: u32, // 维持保证金比例（当保证金低于此比例时可能触发强制清算）
    // 该Market是否完成初始化标志
    pub initialized: u8,
    pub padding0: [u8; 3],
    pub padding1: [u128; 4],
}

impl Market {
    pub fn is_initialized(&self) -> bool {
        if self.initialized == 1 {
            true
        } else {
            false
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
#[repr(u8)]
pub enum OracleSource {
    Pyth,
    Switchboard,
}

unsafe impl Zeroable for OracleSource {}
unsafe impl Pod for OracleSource {}

#[zero_copy]
pub struct AMM {
    pub base_asset_reserve: u128,  // base资产储备量
    pub quote_asset_reserve: u128, // quote资产储备量
    pub sqrt_k: u128,              // 恒定乘积公式中的K值(√K)，用于x*y=k类AMM
    // 资金费率与重新锚定
    pub cumulative_repeg_rebate_long: u128, // 多头累计重新锚定返利
    pub cumulative_repeg_rebate_short: u128, // 空头累计重新锚定返利
    pub cumulative_funding_rate_long: u128, // 多头累计资金费率
    pub cumulative_funding_rate_short: u128, // 空头累计资金费率
    pub last_funding_rate: i128,            // 最近的资金费率
    pub last_funding_rate_ts: i64,          // 最近更新资金费率的时间戳
    pub funding_period: i64,                // 资金费率计算周期
    pub peg_multiplier: u128,               // 锚定乘数，用于调整AMM价格与目标价格的偏差
    // fee相关
    pub total_fee: u128,                     // 累计总费用
    pub total_fee_minus_distributions: u128, // 总费用减去分配部分
    pub total_fee_withdrawn: u128,           // 已提取的总费用
    // 交易参数
    pub minimum_base_asset_trade_size: u128, // base资产最小交易量
    pub mininum_quote_asset_trade_size: u128, // quote资产最小交易量
    // 标记价格
    pub last_mark_price_twap: u128,   // 最近的标记价格的时间加权平均
    pub last_mark_price_twap_ts: i64, // 最近更新标记价格TWAP的时间戳
    // 预言机
    pub last_oracle_price_twap_ts: i64, // 最近更新预言机TWAP的时间戳
    pub last_oracle_price_twap: i128,   // 最近的预言机价格的时间加权平均
    pub oracle: Pubkey,                 // oracle地址
    pub last_oracle_price: i128,        // 最新的预言机价格
    pub base_spread: u16,               // 基础点差(以基点表示)
    pub oracle_source: OracleSource,    // 预言机类型
    // pub pyth_feed_id: [u8; 32],         //查所需的feed id
    pub padding: [u8; 13],
}

impl AMM {
    // 计算当前AMM池base的标记价格（经过peg调整后）
    pub fn mark_price(&self) -> ClearingHouseResult<u128> {
        amm::calculate_price(
            self.quote_asset_reserve,
            self.base_asset_reserve,
            self.peg_multiplier,
        )
    }

    // 从pyth中获取价格信息
    pub fn get_pyth_price(
        &self,
        price_oracle: &AccountInfo,
        clock_slot: u64,
    ) -> ClearingHouseResult<OraclePriceData> {
        let pyth_price_data = price_oracle
            .try_borrow_data()
            .map_err(|_| Errors::FailToLoadOracle)?;
        let price_update_v2 = PriceUpdateV2::deserialize(&mut &pyth_price_data[8..])
            .map_err(|_| Errors::FailToDeserialize)?;
        // 从pyth取到的价格
        let pyth_price = cast_to_i128(price_update_v2.price_message.price)?;
        // 从pyth取到的价格的置信区间
        let pyth_price_conf = cast_to_u128(price_update_v2.price_message.conf)?;
        // 从pyth取到的价格精度
        let pyth_price_precision =
            10u128.pow(price_update_v2.price_message.exponent.unsigned_abs());

        // 为了将pyth喂上来的价格精度统一到市场价格精度，pyth_scale_mul为需要扩大的倍数，pyth_scale_div为需要缩小的倍数
        let mut pyth_scale_mul = 1;
        let mut pyth_scale_div = 1;
        if pyth_price_precision > MARK_PRICE_PRECISION {
            // 如果pyth的给的价格精度>MARK_PRICE_PRECISION，
            // 那么pyth价格精度需要缩小pyth_scale_div=pyth_price_precision/MARK_PRICE_PRECISION
            pyth_scale_div = pyth_price_precision
                .checked_div(MARK_PRICE_PRECISION)
                .ok_or_else(math_error!())?;
        } else {
            // 如果pyth的给的价格精度<=MARK_PRICE_PRECISION，
            // 那么pyth价格精度需要扩大pyth_scale_div=MARK_PRICE_PRECISION/pyth_price_precision
            pyth_scale_mul = MARK_PRICE_PRECISION
                .checked_div(pyth_price_precision)
                .ok_or_else(math_error!())?;
        }

        // 被提升到市场价格精度的价格，类型为i28
        let pyth_price_scaled = pyth_price
            // pyth_scale_mul和pyth_scale_div从u128变成i128
            .checked_mul(cast(pyth_scale_mul)?)
            .ok_or_else(math_error!())?
            .checked_div(cast(pyth_scale_div)?)
            .ok_or_else(math_error!())?;

        // 从pyth取到的价格的置信区间也提升到市场价格精度
        let pyth_price_conf_scaled = pyth_price_conf
            // pyth_scale_mul和pyth_scale_div从u128变成i128
            .checked_mul(pyth_scale_mul)
            .ok_or_else(math_error!())?
            .checked_div(pyth_scale_div)
            .ok_or_else(math_error!())?;

        // pyth价格发布的slot与当前slot的差值
        let pyth_price_delay = cast_to_i64(clock_slot)?
            .checked_sub(cast(price_update_v2.posted_slot)?)
            .ok_or_else(math_error!())?;

        Ok(OraclePriceData {
            price: pyth_price_scaled,
            confidence: pyth_price_conf_scaled,
            delay: pyth_price_delay,
            // 对于pyth的价格，该值永远为true
            has_sufficient_number_of_data_points: true,
        })
    }

    // 从pyth中获取ema价格
    pub fn get_pyth_ema_price(&self, price_oracle: &AccountInfo) -> ClearingHouseResult<i128> {
        let pyth_price_data = price_oracle
            .try_borrow_data()
            .map_err(|_| Errors::FailToLoadOracle)?;
        let price_update_v2 = PriceUpdateV2::deserialize(&mut &pyth_price_data[8..])
            .map_err(|_| Errors::FailToDeserialize)?;
        // 从pyth取到的ema价格
        let pyth_ema_price = cast_to_i128(price_update_v2.price_message.ema_price)?;
        let pyth_price_precision =
            10u128.pow(price_update_v2.price_message.exponent.unsigned_abs());

        // 为了将pyth喂上来的价格精度统一到市场价格精度，pyth_scale_mul为需要扩大的倍数，pyth_scale_div为需要缩小的倍数
        let mut pyth_scale_mul = 1;
        let mut pyth_scale_div = 1;
        if pyth_price_precision > MARK_PRICE_PRECISION {
            // 如果pyth的给的价格精度>MARK_PRICE_PRECISION，
            // 那么pyth价格精度需要缩小pyth_scale_div=pyth_price_precision/MARK_PRICE_PRECISION
            pyth_scale_div = pyth_price_precision
                .checked_div(MARK_PRICE_PRECISION)
                .ok_or_else(math_error!())?;
        } else {
            // 如果pyth的给的价格精度<=MARK_PRICE_PRECISION，
            // 那么pyth价格精度需要扩大pyth_scale_div=MARK_PRICE_PRECISION/pyth_price_precision
            pyth_scale_mul = MARK_PRICE_PRECISION
                .checked_div(pyth_price_precision)
                .ok_or_else(math_error!())?;
        }

        let pyth_ema_price_scaled = pyth_ema_price
            .checked_mul(cast(pyth_scale_mul)?)
            .ok_or_else(math_error!())?
            .checked_div(cast(pyth_scale_div)?)
            .ok_or_else(math_error!())?;

        Ok(pyth_ema_price_scaled)
    }
}

pub struct OraclePriceData {
    pub price: i128, // 从预言机获取的资产价格（已根据MARK_PRICE_PRECISION提升了精度）
    pub confidence: u128, // 价格的可信度/置信区间，值越小表示价格越可信
    pub delay: i64,  // 表示价格数据的延迟时间(以slot为单位)，即当前slot-价格数据最后更新的slot
    pub has_sufficient_number_of_data_points: bool, // 表示预言机数据是否有足够的数据点支持。对于Switchboard而言，检查确认的轮次是否达到最小预言机结果数。对于Pyth预言机，这个值总是true
}
