use anchor_lang::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("Clearing house not collateral vault owner")]
    InvalidCollateralVaultAuthority,
    #[msg("Clearing house not insurance vault owner")]
    InvalidInsuranceVaultAuthority,
    #[msg("Clearing house histories already initialized")]
    HistoriesAllInitialized,
    #[msg("Clearing house order state already initialized")]
    OrderStateAlreadyInitialized,
    #[msg("Market index already initialized")]
    MarketIndexAlreadyInitialized,
    #[msg("Invalid initial peg")]
    InvalidInitialPeg,
    #[msg("Math error")]
    MathError,
    #[msg("Conversion to u128/u64 failed with an overflow or underflow")]
    BnConversionError,
    #[msg("Fail to load oracle")]
    FailToLoadOracle,
    #[msg("Fail to deserialize")]
    FailToDeserialize,
    #[msg("Fail to cast")]
    FailToCast,
    #[msg("Invalid margin ratio")]
    InvalidMarginRatio,
    #[msg("Fail to find whitelist token")]
    FailToFindWhitelistToken,
    #[msg("Invalid whitelist token")]
    InvalidWhitelistToken,
    #[msg("No balance")]
    WhitelistTokenNoBalance,
}

// #[macro_export]：使宏可以被其他模块通过`use crate::math_error;`导入
#[macro_export]
// math_error宏返回一个闭包，该闭包会打印详细的错误msg（包含宏被调用处的代码位置）并返回 MathError
macro_rules! math_error {
    () => {{    // 无参数宏
        || {    // 返回一个闭包
            // 从当前crate的Errors模块中获取MathError错误类型
            // 注：$crate 是一个特殊变量，指向当前crate的根路径（避免硬编码crate名称）
            let err = $crate::errors::Errors::MathError;
            // 注：file!() 和 line!()为Rust内置宏，用于获取代码位置。会指向宏调用处，而非宏定义处
            msg!("Error {} thrown at {}:{}", err, file!(), line!());
            err
        }
    }};
}
