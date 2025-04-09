use super::bn::ClearingHouseResult;
use crate::errors::Errors;

pub fn cast<T: TryInto<U>, U>(t: T) -> ClearingHouseResult<U> {
    t.try_into().map_err(|_| Errors::FailToCast)
}

// 将T类型安全转换成i128
pub fn cast_to_i128<T: TryInto<i128>>(t: T) -> ClearingHouseResult<i128> {
    cast(t)
}

// 将T类型安全转换成u128
pub fn cast_to_u128<T: TryInto<u128>>(t: T) -> ClearingHouseResult<u128> {
    cast(t)
}

// 将T类型安全转换成i64
pub fn cast_to_i64<T: TryInto<i64>>(t: T) -> ClearingHouseResult<i64> {
    cast(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cast_to_i128() {
        let mut t = i128::MAX as u128;
        assert_eq!(cast_to_i128(t).unwrap(), i128::MAX);

        t = i128::MAX as u128 + 1;
        assert!(cast_to_i128(t).is_err());
    }

    #[test]
    fn test_cast_to_i64() {
        let mut t = i64::MAX as u128;
        assert_eq!(cast_to_i64(t).unwrap(), i64::MAX);

        t = i64::MAX as u128 + 1;
        assert!(cast_to_i64(t).is_err());
    }
}
