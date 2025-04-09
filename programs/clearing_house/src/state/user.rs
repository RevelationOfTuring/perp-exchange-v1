use anchor_lang::prelude::*;
use static_assertions::const_assert_eq;
use std::mem::size_of;

#[account]
#[repr(C)]
pub struct User {
    pub authority: Pubkey,                       // 该账户的所有者
    pub collateral: u128,                        // 用户当前存入的抵押品总额
    pub cumculative_deposits: i128,              // 用户历史累计存款（可能为负值，表示净提款）
    pub total_fee_paid: u64,                     // 用户累计支付的手续费
    pub total_fee_rebate: u64,                   // 用户累计获得的手续费返还
    pub total_token_discount: u128,              // 平台代币折扣累计
    pub total_referral_reward: u128,             // 推荐他人获得的奖励总额
    pub total_referee_discount: u128,            // 被推荐用户累计获得的折扣
    pub positons: Pubkey, // 关联的 UserPositions 账户地址（存储用户持仓细节）
    pub settled_position_value: u128, // 已结算持仓的价值（平仓时锁定）
    pub collateral_claimed: u64, // 用户已提取的抵押品金额
    pub last_collateral_available_to_claim: u64, // 最后一次可提取的抵押品余额
    pub forgo_position_settlement: u8, // 标志位，表示用户是否放弃持仓结算
    pub has_settled_position: u8, // 标志位，表示是否有已结算的持仓
    pub padding: [u8; 14],
}

const_assert_eq!(size_of::<User>(), 208);

#[account(zero_copy)]
pub struct UserPositions {
    // user key
    pub user: Pubkey,
    pub positions: [MarketPosition; 5],
}

const_assert_eq!(size_of::<UserPositions>(), 1072);

#[zero_copy]
pub struct MarketPosition {
    // 市场的唯一标识符
    pub market_index: u64,
    // 最后一次资金费率更新的时间戳
    pub last_funding_rate_ts: i64,
    // 用户在该市场的base资产持仓量。
    // 正数表示多头头寸（买入并持有基础资产），负数表示空头头寸（借入并卖出基础资产），0表示无持仓
    pub base_asset_amount: i128,
    // 用户在该市场的quote资产持仓量
    // 正数表示多头头寸（买入并持有基础资产），负数表示空头头寸（借入并卖出基础资产），0表示无持仓
    pub quote_asset_amount: i128,
    // 最后一次更新头寸时的累计资金费率
    pub last_cumulative_funding_rate: i128,
    // 最后一次更新头寸时的重新锚定（repeg）返利累计值（某些协议会调整合约价格锚定，并补偿用户）
    pub last_cumulative_repeg_rebate: i128,
    // 用户在该市场的未成交订单数量
    pub open_orders: u128,
    // 预留
    pub padding: [u128; 7],
}

impl MarketPosition {
    // 检查当前MarketPosition是否属于指定的market_index，并且当前头寸是活跃的（即有未平仓头寸或未成交的订单）
    pub fn is_for(&self, market_index: u64) -> bool {
        self.market_index == market_index && (self.is_open_position() || self.has_open_order())
    }

    // 检查当前MarketPosition是否有未成交的订单
    pub fn has_open_order(&self) -> bool {
        self.open_orders != 0
    }

    // 检查当前MarketPosition是否有未平仓头寸
    pub fn is_open_position(&self) -> bool {
        self.base_asset_amount != 0
    }

    // 检查当前MarketPosition是否可用（即没有未平仓头寸且没有未成交订单）
    pub fn is_available(&self) -> bool {
        !self.is_open_position() && !self.has_open_order()
    }
}
