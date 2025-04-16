use crate::math_error;
use anchor_lang::prelude::*;

use crate::math::{
    bn::{ClearingHouseResult, U192},
    constant::*,
};

// quote_asset_reserve/base_asset_reserve*(peg_multiplier/PEG_PRECISION) -> 最后提升到MARK_PRICE_PRECISION精度
pub fn calculate_price(
    quote_asset_reserve: u128,
    base_asset_reserve: u128,
    // 锚定乘数
    peg_multiplier: u128,
) -> ClearingHouseResult<u128> {
    // 1. 先计算经过peg_multiplier调整后的价格p：(peg_multiplier/PEG_PRECISION) * quote_asset_reserve/base_asset_reserve
    // 2. 再将价格提升到价格精度: p*MARK_PRICE_PRECISION
    let peg_quote_asset_amount = quote_asset_reserve
        .checked_mul(peg_multiplier)
        .ok_or_else(math_error!())?;

    U192::from(peg_quote_asset_amount)
        .checked_mul(U192::from(PRICE_TO_PEG_PRECISION_RATIO))
        .ok_or_else(math_error!())?
        .checked_div(U192::from(base_asset_reserve))
        .ok_or_else(math_error!())?
        .try_to_u128()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_calculate_price() {
        assert_eq!(
            calculate_price(1000, 1000, 1000).unwrap(),
            MARK_PRICE_PRECISION
        );
    }
}
