use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use static_assertions::const_assert_eq;

#[account(zero_copy)]
pub struct DepositHistory {
    head: u64,
    padding: [u8; 8],
    deposit_records: [DepositRecord; 1024],
}

const_assert_eq!(std::mem::size_of::<DepositHistory>(), 147472);

#[repr(u8)]
#[derive(Clone, Copy, AnchorDeserialize, AnchorSerialize)]
pub enum DepositDirection {
    Deposit,
    Withdraw,
}

unsafe impl Zeroable for DepositDirection {}
unsafe impl Pod for DepositDirection {}

#[zero_copy]
pub struct DepositRecord {
    pub ts: i64,                          // 时间戳
    pub amount: u64,                      // 本次操作的金额
    pub record_id: u128,                  // 唯一标识符，用于区分不同记录
    pub user_authority: Pubkey,           // 用户授权公钥（执行交易的地址）
    pub user: Pubkey,                     // 用户账户公钥
    pub collateral_before: u128,          // 操作前的抵押品总额
    pub cumulative_deposits_before: i128, // 操作前的累计存款总额
    pub direction: DepositDirection,      // 操作方向（存款或取款）
    pub padding: [u8; 15],
}

impl DepositHistory {
    // 增添deposit_record
    pub fn append(&mut self, deposit_record: DepositRecord) {
        self.deposit_records[Self::index(self.head)] = deposit_record;
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
        let pre_record = &self.deposit_records[Self::index(pre_record_id)];
        pre_record.record_id + 1
    }
}
