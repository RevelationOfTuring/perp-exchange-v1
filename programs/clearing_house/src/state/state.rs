use anchor_lang::prelude::*;
use static_assertions::const_assert_eq;
use std::mem::size_of;

#[account(zero_copy)]
pub struct State {
    pub exchange_paused: u8,                  // 整个exchange是否暂停交易
    pub funding_paused: u8,                   // 资金费率计算是否暂停
    pub admin_controls_prices: u8,            // 管理员是否控制价格(紧急情况下)
    pub collateral_vault_authority_nonce: u8, // 生成collateral_vault_authority的bump值
    pub insurance_vault_authority_nonce: u8,  // 生成insurance_vault_authority的bump值
    pub padding0: [u8; 11],

    pub admin: Pubkey,
    pub collateral_mint: Pubkey,            // 抵押品token的mint地址
    pub collateral_vault: Pubkey,           // 本program的存储抵押品token的token account地址
    pub collateral_vault_authority: Pubkey, // collateral_vault的authority账户(pda)
    pub deposit_history: Pubkey,            // 存款历史记录账户地址
    pub trade_history: Pubkey,              // 交易历史记录账户地址
    pub funding_payment_history: Pubkey,    // 资金支付历史记录账户地址
    pub funding_rate_history: Pubkey,       // 资金费率历史记录账户地址
    pub liquidation_history: Pubkey,        // 清算历史记录账户地址
    pub curve_history: Pubkey,              // 曲线历史记录账户地址(可能指AMM曲线参数)
    pub insurance_vault: Pubkey,            // 本program的存储保险金token的token account地址
    pub insurance_vault_authority: Pubkey,  // insurance_vault的authority账户(pda)
    pub markets: Pubkey,                    // 全部市场账户地址
    pub margin_ratio_initial: u128,         // 初始保证金比例
    pub margin_ratio_maintenance: u128,     // 维持保证金比例
    pub margin_ratio_partial: u128,         // 部分清算保证金比例
    pub partial_liquidation_close_percentage_numerator: u128, // 当触发部分清算时，应平仓头寸的比例的分子
    pub partial_liquidation_close_percentage_denominator: u128, // 当触发部分清算时，应平仓头寸的比例的分母
    pub partial_liquidation_penalty_percentage_numberator: u128, // 部分清算时收取的惩罚费率的分子
    pub partial_liquidation_penalty_percentage_denominator: u128, // 部分清算时收取的惩罚费率的分母
    pub full_liquidation_penalty_percentage_numerator: u128,    // 完全清算时收取的惩罚费率的分子
    pub full_liquidation_penalty_percentage_denominator: u128,  // 完全清算时收取的惩罚费率的分母
    pub partial_liquidation_liquidator_share_denominator: u128, // 部分清算时清算人份额分母
    pub full_liquidation_liquidator_share_denominator: u128,    // 完全清算时清算人份额分母
    pub fee_structure: FeeStructure,                            // fee结构
    pub whitelist_mint: Pubkey,                                 // 白名单代币的mint地址
    pub discount_mint: Pubkey,                                  // 用于折扣的代币mint地址
    pub oracle_guard_rails: OracleGuardRails,                   // 预言机保护机制
    pub max_deposit: u128,                                      // 最大存款限额
    pub extended_curve_history: Pubkey,                         // 扩展的曲线历史记录账户地址
    pub order_state: Pubkey,                                    // 订单状态账户地址
    // Upgrade ability
    pub padding1: [u128; 4],
}

const_assert_eq!(size_of::<State>(), 1200);

// Oracle防护栏（防护机制）
#[zero_copy]
pub struct OracleGuardRails {
    pub price_divergence: PriceDivergenceGuardRails, // 价格偏离保护(标记价格与预言机价格的最大偏离比例)
    pub validity: ValidityGuardRails,                // 数据有效性检查
    pub use_for_liquidations: u8,                    // 是否在清算时启用这些保护
    pub padding: [u8; 15],
}

// 价格背离防护栏
#[zero_copy]
pub struct PriceDivergenceGuardRails {
    pub mark_oracle_divergence_numerator: u128, // 标记价格与预言机价格的最大偏离比例分子
    pub mark_oracle_divergence_denominator: u128, // 标记价格与预言机价格的最大偏离比例分母
}

// 有效性防护机制
#[zero_copy]
pub struct ValidityGuardRails {
    pub confidence_interval_max_size: u128, // 置信区间最大宽度
    pub too_volatile_ratio: i128,           // 价格波动率阈值
    pub slots_before_stable: i64, // 预言机数据过期阈值（按Slot数）: 预言机数据若超过此Slot数未更新，视为陈旧
    pub padding: [u8; 8],
}

// fee结构
#[zero_copy]
pub struct FeeStructure {
    pub fee_numerator: u128,                      // 基础fee分子
    pub fee_denominator: u128,                    // 基础fee分母
    pub discount_token_tiers: DiscountTokenTiers, // 持币折扣分级(4个层级)，每个等级有最低余额要求和折扣比例
    pub referral_discount: ReferralDiscount,      // 推荐奖励
}

#[zero_copy]
pub struct DiscountTokenTiers {
    pub first_tier: DiscountTokenTier,  // 第1档
    pub second_tier: DiscountTokenTier, // 第2档
    pub third_tier: DiscountTokenTier,  // 第3档
    pub fourth_tier: DiscountTokenTier, // 第4档
}

#[zero_copy]
pub struct DiscountTokenTier {
    pub discount_numerator: u128,   // 折扣率分子
    pub discount_denominator: u128, // 折扣率分母
    pub minimun_balance: u64,       // 该档位最低持币量要求
    pub padding: [u8; 8],
}

#[zero_copy]
pub struct ReferralDiscount {
    pub referral_reward_numerator: u128,    // 推荐人奖励分子
    pub referral_reward_denominator: u128,  // 推荐人奖励分母
    pub referee_discount_numerator: u128,   // 被推荐人折扣分子
    pub referee_discount_denominator: u128, // 被推荐人折扣分母
}
