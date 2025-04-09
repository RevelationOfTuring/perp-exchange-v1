use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use static_assertions::const_assert_eq;

use crate::state::user_orders::Order;

#[account(zero_copy)]
pub struct OrderHistory {
    head: u64, // 作为循环缓冲区的指针，指示下一个记录应该写入的位置
    _padding: [u8; 8],
    pub last_order_id: u128, // 最近一个订单的全局唯一ID
    order_records: [OrderRecord; 1024],
}

const_assert_eq!(std::mem::size_of::<OrderHistory>(), 458784);

#[zero_copy]
pub struct OrderRecord {
    pub ts: i64,             // 交易时间戳
    pub action: OrderAction, // 订单操作类型（枚举）
    pub padding: [u8; 7],
    pub record_id: u128,                  // 该条记录的唯一ID
    pub user: Pubkey,                     // 订单所属用户的地址
    pub authority: Pubkey,                // 执行该订单操作的授权账户
    pub order: Order,                     // 订单的具体信息
    pub filler: Pubkey,                   // 订单的成交者（做市商或匹配引擎）
    pub trade_record_id: u128,            // 关联的trader reocord ID
    pub base_asset_amount_filled: u128,   // 已成交的base资产数量
    pub quote_asset_amount_filled: u128,  // 已成交的quote资产数量
    pub fee: i128,                        // 交易手续费（可能是正或负，例如返佣情况）
    pub filler_reward: u128,              // 做市商（订单填充者）的奖励金额
    pub quote_asset_amount_surplus: u128, // quote资产的剩余金额（可能用于部分成交或滑点计算）
}

impl OrderHistory {
    // 增添order_record
    pub fn append(&mut self, order_record: OrderRecord) {
        self.order_records[Self::index(self.head)] = order_record;
        self.head = (self.head + 1) % 1024;
    }

    // 将u64安全转为usize
    pub fn index(counter: u64) -> usize {
        std::convert::TryInto::try_into(counter).unwrap()
    }

    // 下一个record的record_id
    // 注： self.head会在0~1023之间来回递增，而每个record.record_id一直单向递增
    pub fn next_record_id(&self) -> u128 {
        let pre_record_id = if self.head == 0 { 1023 } else { self.head - 1 };
        let pre_record = &self.order_records[Self::index(pre_record_id)];
        pre_record.record_id + 1
    }

    pub fn next_order_id(&mut self) -> u128 {
        let next_order_id = self.last_order_id + 1;
        self.last_order_id = next_order_id;
        next_order_id
    }
}

#[derive(Clone, Copy, AnchorSerialize, AnchorDeserialize)]
#[repr(u8)]
pub enum OrderAction {
    Place,  // 下单
    Cancel, // 取消订单
    Fill,   // 吃单
    Expire, // 过期
}

unsafe impl Zeroable for OrderAction {}
unsafe impl Pod for OrderAction {}
