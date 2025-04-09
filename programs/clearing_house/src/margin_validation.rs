use crate::{
    errors::Errors,
    math::{
        bn::ClearingHouseResult,
        constant::{MAXIMUM_MARGIN_RATIO, MINIMUM_MARGIN_RATIO},
    },
};

pub fn margin_validation(
    // 初始保证金率（如 2000 = 20%），控制开仓最低抵押率
    margin_ratio_initial: u32,
    // 部分平仓保证金率，触发部分清算的阈值
    margin_ratio_partial: u32,
    // 维持保证金率，触发全额清算的阈值
    margin_ratio_maintenance: u32,
) -> ClearingHouseResult {
    // 如果margin_ratio_initial不处于[MINIMUM_MARGIN_RATIO,MAXIMUM_MARGIN_RATIO]区间，报错
    if !(MINIMUM_MARGIN_RATIO..=MAXIMUM_MARGIN_RATIO).contains(&margin_ratio_initial) {
        return Err(Errors::InvalidMarginRatio);
    }

    // 如果初始保证金率小于部分平仓保证金率，报错
    if margin_ratio_initial < margin_ratio_partial {
        return Err(Errors::InvalidMarginRatio);
    }

    // 如果margin_ratio_partial不处于[MINIMUM_MARGIN_RATIO,MAXIMUM_MARGIN_RATIO]区间，报错
    if !(MINIMUM_MARGIN_RATIO..=MAXIMUM_MARGIN_RATIO).contains(&margin_ratio_partial) {
        return Err(Errors::InvalidMarginRatio);
    }

    // 如果部分平仓保证金率小于维持保证金率，报错
    if margin_ratio_partial < margin_ratio_maintenance {
        return Err(Errors::InvalidMarginRatio);
    }

    // 如果margin_ratio_maintenance不处于[MINIMUM_MARGIN_RATIO,MAXIMUM_MARGIN_RATIO]区间，报错
    if !(MINIMUM_MARGIN_RATIO..=MAXIMUM_MARGIN_RATIO).contains(&margin_ratio_maintenance) {
        return Err(Errors::InvalidMarginRatio);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_margin_validation() {
        assert!(margin_validation(300, 250, 200).is_ok());
        assert!(margin_validation(199, 250, 200).is_err());
        assert!(margin_validation(300, 199, 200).is_err());
        assert!(margin_validation(300, 250, 199).is_err());
        assert!(margin_validation(249, 250, 200).is_err());
        assert!(margin_validation(300, 200, 201).is_err());
    }
}
