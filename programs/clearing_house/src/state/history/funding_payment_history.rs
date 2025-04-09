use anchor_lang::prelude::*;
use static_assertions::const_assert_eq;

#[account(zero_copy)]
pub struct FundingPaymentHistory {
    // 作为循环缓冲区的指针，指示下一个记录应该写入的位置
    head: u64,
    padding: [u8; 8],
    // 存储资金费率支付记录的实际数据
    funding_payment_records: [FundingPaymentRecord; 1024],
}

const_assert_eq!(std::mem::size_of::<FundingPaymentHistory>(), 196624);

#[zero_copy]
pub struct FundingPaymentRecord {
    pub ts: i64,                            // 时间戳（记录创建时间）
    pub market_index: u64,                  // 市场索引
    pub record_id: u128,                    // 记录的唯一标识符
    pub user_authority: Pubkey,             // 用户authority地址
    pub user: Pubkey,                       // 用户账户地址
    pub funding_payment: i128, // 资金费率支付金额（使用有符号128位整数表示正负支付方向）
    pub base_asset_amount: i128, // 基础资产数量（用户持仓量）
    pub amm_cumulative_funding_long: i128, // AMM累计多头资金费率
    pub amm_cumulative_funding_short: i128, // AMM累计空头资金费率
    pub user_last_cumulative_funding: i128, // 用户上次累计资金费率
    pub user_last_funding_rate_ts: i64, // 用户上次资金费率更新时间戳
    pub padding: [u8; 8],
}

impl FundingPaymentHistory {
    // 增添funding_payment_record
    pub fn append(&mut self, funding_payment_record: FundingPaymentRecord) {
        self.funding_payment_records[Self::index(self.head)] = funding_payment_record;
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
        let pre_record = &self.funding_payment_records[Self::index(pre_record_id)];
        pre_record.record_id + 1
    }
}
