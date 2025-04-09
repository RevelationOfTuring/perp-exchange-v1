use anchor_lang::prelude::*;
use static_assertions::const_assert_eq;

#[account(zero_copy)]
pub struct LiquidationHistory {
    // 作为循环缓冲区的指针，指示下一个记录应该写入的位置
    head: u64,
    padding: [u8; 8],
    liquidation_records: [LiquidationRecord; 1024],
}

const_assert_eq!(std::mem::size_of::<LiquidationHistory>(), 262160);

#[zero_copy]
pub struct LiquidationRecord {
    pub record_id: u128,               // 记录的唯一标识符
    pub user_authority: Pubkey,        // 用户授权公钥（执行交易的地址）
    pub user: Pubkey,                  // 被清算的用户账户公钥
    pub liquidator: Pubkey,            // 执行清算操作的账户公钥
    pub base_asset_value: u128,        // base资产总值（清算前）
    pub base_asset_value_closed: u128, // 被清算的base资产价值
    pub liquidation_fee: u128,         // 总清算费用
    pub fee_to_liquidator: u64,        // 支付给清算方的费用
    pub fee_to_insurance_fund: u64,    // 支付给保险基金的费用
    pub total_collateral: u128,        // 总抵押品价值
    pub collateral: u128,              // 被清算的抵押品数量
    pub unrealized_pnl: i128,          // 未实现盈亏（可为正或负）
    pub margin_ratio: u128,            // 保证金比率（表示账户的风险水平）
    pub ts: i64,                       // 时间戳（记录创建时间）
    pub partial: u8,                   // 是否为部分清算（1=部分，0=完全）
    pub padding: [u8; 7],
}

impl LiquidationHistory {
    // 增添liquidation_record
    pub fn append(&mut self, liquidation_record: LiquidationRecord) {
        self.liquidation_records[Self::index(self.head)] = liquidation_record;
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
        let pre_record = &self.liquidation_records[Self::index(pre_record_id)];
        pre_record.record_id + 1
    }
}
