use anchor_lang::prelude::*;
use static_assertions::const_assert_eq;

#[account(zero_copy)]
pub struct CurveHistory {
    head: u64,
    padding: [u8; 8],
    curve_records: [CurveRecord; 1024],
}

const_assert_eq!(std::mem::size_of::<CurveHistory>(), 311312);

#[zero_copy]
pub struct CurveRecord {
    pub ts: i64,                             // 时间戳
    pub market_index: u64,                   // 市场索引标识
    pub record_id: u128,                     // 单调递增的唯一记录ID
    pub peg_multiplier_before: u128,         // 交易前的锚定乘数
    pub base_asset_reserve_before: u128,     // 交易前的base资产储备量
    pub quote_asset_reserve_before: u128,    // 交易前的quote资产储备量
    pub sqrt_k_before: u128,                 // 交易前的√(k)值(k=x*y)
    pub peg_multiplier_after: u128,          // 交易后的锚定乘数
    pub base_asset_reserve_after: u128,      // 交易后的base资产储备量
    pub quote_asset_reserve_after: u128,     // 交易后的quote资产储备量
    pub sqrt_k_after: u128,                  // 交易后的√(k)值
    pub base_asset_amount_long: u128,        // 多头仓位总量
    pub base_asset_amount_short: u128,       // 空头仓位总量
    pub base_asset_amount: i128,             // 净仓位(可正可负)
    pub open_interest: u128,                 // 未平仓合约总量
    pub total_fee: u128,                     // 总手续费
    pub total_fee_minus_distributions: u128, // 总手续费减去分配部分
    pub adjustment_cost: i128,               // 调整成本(正表示收入，负表示支出)
    pub oracle_price: i128,                  // 预言机价格
    pub trade_record: u128,                  // 关联的交易记录ID
}

impl CurveHistory {
    // 增添curve_record
    pub fn append(&mut self, curve_record: CurveRecord) {
        self.curve_records[Self::index(self.head)] = curve_record;
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
        let pre_record = &self.curve_records[Self::index(pre_record_id)];
        pre_record.record_id + 1
    }
}
