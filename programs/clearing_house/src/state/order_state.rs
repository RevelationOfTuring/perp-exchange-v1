use anchor_lang::prelude::*;
use static_assertions::const_assert_eq;

#[account]
#[repr(C)]
pub struct OrderState {
    pub order_history: Pubkey, // 存储order历史记录的账户
    pub order_filler_reward_structure: OrderFillerRewardStructure, // order填充者的奖励结构
    pub min_order_quote_asset_amount: u128, // 订单成功放置所需的最小quote资产金额估计值
}

const_assert_eq!(std::mem::size_of::<OrderState>(), 96);

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct OrderFillerRewardStructure {
    pub reward_numerator: u128,              // 奖励计算的分子
    pub reward_denominator: u128,            // 奖励计算的分母
    pub time_based_reward_lower_bound: u128, // 基于时间的奖励下限，确保即使订单金额很小，填充者也能获得最低奖励。防止微小订单的奖励过低而无人愿意填充
}
