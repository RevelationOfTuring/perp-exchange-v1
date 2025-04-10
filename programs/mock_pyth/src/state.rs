use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update;
use std::ops::{Deref, DerefMut};

// 由于price_update::PriceUpdateV2的ower()方法返回的是rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ，而非本program地址
// 所以无法通过anchor的校验。这里做一层封装PriceUpdate，它在底层与price_update::PriceUpdateV2完全一致，且其owner是本program
#[account]
pub struct PriceUpdate(price_update::PriceUpdateV2);

impl PriceUpdate {
    pub const LEN: usize = price_update::PriceUpdateV2::LEN;
}

// 通过实现 Deref trait 将PriceUpdate解引用为price_update::PriceUpdateV2，从而“继承”price_update::PriceUpdateV2原有&self方法
impl Deref for PriceUpdate {
    type Target = price_update::PriceUpdateV2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// 通过实现 DerefMut trait 将PriceUpdate解引用为price_update::PriceUpdateV2，从而“继承”price_update::PriceUpdateV2原有&mut self方法
impl DerefMut for PriceUpdate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
