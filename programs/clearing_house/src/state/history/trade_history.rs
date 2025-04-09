use anchor_lang::prelude::*;
use static_assertions::const_assert_eq;

use crate::PositionDirection;

#[account(zero_copy)]
pub struct TradeHistory {
    // 作为循环缓冲区的指针，指示下一个交易记录应该写入的位置
    head: u64,
    padding: [u8; 8],
    trade_record: [TradeRecord; 1024],
}

const_assert_eq!(std::mem::size_of::<TradeHistory>(), 262160);

#[zero_copy]
pub struct TradeRecord {
    pub ts: i64,                          // 交易时间戳
    pub market_index: u64,                // 市场索引
    pub record_id: u128,                  // 交易记录的唯一ID
    pub user_authority: Pubkey,           // 执行交易的用户钱包地址
    pub user: Pubkey,                     // 用户在协议中的账户地址
    pub base_asset_amount: u128,          // base资产交易数量
    pub quote_asset_amount: u128,         // quote资产交易数量
    pub mark_price_before: u128,          // 交易前的标记价格
    pub mark_price_after: u128,           // 交易后的标记价格
    pub fee: i128,                        // 交易手续费(有符号可能表示手续费返还)
    pub quote_asset_amount_surplus: u128, // quote资产盈余/差额(可能用于滑价补偿)
    pub referee_discount: u128,           // 推荐人折扣金额
    pub token_discount: u128,             // 使用平台代币支付的折扣金额
    pub oracle_price: i128,               // 交易时的预言机价格(用于比较标记价格)
    pub liquidation: u8,                  // 是否是清算交易(1表示这是强制平仓)
    pub direction: PositionDirection,     // 交易方向
    pub padding: [u8; 14],
}

impl TradeHistory {
    // 增添trade_record
    pub fn append(&mut self, trade_record: TradeRecord) {
        self.trade_record[Self::index(self.head)] = trade_record;
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
        let pre_record = &self.trade_record[Self::index(pre_record_id)];
        pre_record.record_id + 1
    }
}
