use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::controller::position::PositionDirection;

#[zero_copy]
pub struct Order {
    pub status: OrderStatus,                      // 订单状态
    pub order_type: OrderType,                    // 订单类型
    pub direction: PositionDirection,             // 方向（做多做空）
    pub user_order_id: u8,                        // 用户自定义订单标识（0-255）
    pub reduce_only: u8,                          // 是否仅减仓（不允许增加风险）
    pub post_only: u8,                            // 是否只做Maker（不支付Taker费）
    pub immediate_or_cancel: u8,                  // 是否立即成交否则取消（IOC）
    pub discount_tier: OrderDiscountTier,         // 手续费折扣等级（从无折扣到4级折扣）
    pub trigger_condition: OrderTriggerCondition, // 订单触发条件
    pub padding: [u8; 7],
    pub ts: i64,                         // 订单创建时间戳
    pub market_index: u64,               // 交易对的市场索引（如 BTC/USDC=0）
    pub order_id: u128,                  // 全局唯一订单ID
    pub price: u128,                     // 订单限价（原始价格）
    pub user_base_asset_amount: i128,    // 用户期望的base资产数量（可正负）
    pub quote_asset_amount: u128,        // 实际quote资产数量（绝对值）
    pub base_asset_amount: u128,         // 实际base资产数量（绝对值）
    pub base_asset_amount_filled: u128,  // 已成交的base资产数量
    pub quote_asset_amount_filled: u128, // 已成交的quote资产数量
    pub fee: i128,                       // 手续费（可正可负，负值表示返佣）
    pub trigger_price: u128,             // 触发单的触发价格
    pub referrer: Pubkey,                // 推荐人地址（用于返佣）
    pub oracle_price_offset: i128,       // 相对于预言机价格的偏移量（动态定价）
}

#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize)]
#[repr(u8)]
pub enum OrderStatus {
    Init, // 订单已创建但未开放（如触发单等待条件）
    Open, // 订单可被执行
}

unsafe impl Zeroable for OrderStatus {}
unsafe impl Pod for OrderStatus {}

#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize)]
#[repr(u8)]
pub enum OrderType {
    Market,        // 市价单
    Limit,         // 限价单
    TriggerMarket, // 触发市价单（条件触发后转市价单）
    TriggerLimit,  // 触发限价单（条件触发后转限价单）
}

unsafe impl Zeroable for OrderType {}
unsafe impl Pod for OrderType {}

#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize)]
#[repr(u8)]
pub enum OrderDiscountTier {
    None,   // 订单手续费无折扣
    First,  // 订单手续费1级折扣
    Second, // 订单手续费2级折扣
    Third,  // 订单手续费3级折扣
    Fourth, // 订单手续费4级折扣
}

unsafe impl Zeroable for OrderDiscountTier {}
unsafe impl Pod for OrderDiscountTier {}

#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize)]
#[repr(u8)]
pub enum OrderTriggerCondition {
    Above, // 当市场价格高于触发价时激活
    Below, // 当市场价格低于触发价时激活
}

unsafe impl Zeroable for OrderTriggerCondition {}
unsafe impl Pod for OrderTriggerCondition {}
