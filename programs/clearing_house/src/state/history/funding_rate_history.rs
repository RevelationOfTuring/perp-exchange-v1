use anchor_lang::prelude::*;
use static_assertions::const_assert_eq;

#[account(zero_copy)]
pub struct FundingRateHistory {
    head: u64,
    padding: [u8; 8],
    funding_rate_record: [FundingRateRecord; 1024],
}

const_assert_eq!(std::mem::size_of::<FundingRateHistory>(), 114704);

#[zero_copy]
pub struct FundingRateRecord {
    pub ts: i64,                             // 时间戳
    pub market_index: u64,                   // 市场索引，
    pub record_id: u128,                     // 唯一标识符
    pub funding_rate: i128,                  // 当前资金费率（可为正或负
    pub cumulative_funding_rate_long: i128,  // 多头仓位的累计资金费率
    pub cumulative_funding_rate_short: i128, // 空头仓位的累计资金费率
    pub oracle_price_twap: i128,             // 预言机价格的TWAP（时间加权平均价）
    pub mark_price_twap: u128,               // 标记价格的TWAP（时间加权平均价）
}

impl FundingRateHistory {
    // 增添funding_rate_record
    pub fn append(&mut self, funding_rate_record: FundingRateRecord) {
        self.funding_rate_record[Self::index(self.head)] = funding_rate_record;
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
        let pre_record = &self.funding_rate_record[Self::index(pre_record_id)];
        pre_record.record_id + 1
    }
}
