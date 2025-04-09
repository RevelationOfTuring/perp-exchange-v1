use crate::errors::Errors::{self, BnConversionError};
use anchor_lang::{AnchorDeserialize, AnchorSerialize};
use std::io::{Read, Write};
use std::mem::size_of;
use uint::construct_uint;

// 封装的一个内部Result
// 由于uint::construct_uint宏生成的代码中的Result都是标准库的std::result::Result，所以如果在本文件中use use anchor_lang::prelude::Result会导致编译无法编译
// 而本合约所有业务逻辑的底层都是U192或U256计算，所以这里封装一个基于标准库Result的类型ClearingHouseResult<T, E>，其T为正常逻辑返回值，E为errors.rs中定义的Errors
// 这样，在所有的内部函数的返回值都可以使用ClearingHouseResult<T>（即如果出现Err，一定要返回在自定义的Errors类型），然后在lib.rs中（即instruction出口）中再转换成anchor_lang::prelude::Result
pub type ClearingHouseResult<T = ()> = std::result::Result<T, Errors>;
// 注：上述类型定义中<>内部的T = ()表示如果函数没有具体返回值（仅可能失败），可以直接用ClearingHouseResult（等价于 Result<(), ErrorCode>）

// 为U192和U256实现AnchorSerialize
// 注：推荐使用#[inline]的场景：1. Borsh序列化实现；2. 简单的小型结构体Borsh反序列化（反序列化逻辑仅1-2行，内联后性能提升明显）。复杂的嵌套结构体的Borsh反序列化不推荐使用内联
macro_rules! impl_anchor_serialize_for_bn {
    ($type: ident) => {
        impl AnchorSerialize for $type {
            #[inline]
            fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
                let bytes = self.to_little_endian();
                writer.write_all(&bytes)
            }
        }
    };
}

// 为U192和U256实现AnchorDeserialize
macro_rules! impl_anchor_deserialize_for_bn {
    ($type: ident) => {
        impl AnchorDeserialize for $type {
            #[inline]
            fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
                let mut buf = Vec::with_capacity(size_of::<Self>());
                reader.read_exact(&mut buf)?;
                Ok(Self::from_little_endian(&buf))
            }
        }
    };
}

// 在编译时生成高精度无符号大整数类型U192
// 使用construct_uint生成的类型，其存储方式为栈分配的u64数组
// 192位的无符号整数，参数3表示底层用3个u64存储数据，即64*3=192 bit
construct_uint! {
    pub struct U192(3);
}

impl U192 {
    // U192安全转换为u64，返回Result
    #[inline]
    pub fn try_to_u64(self) -> ClearingHouseResult<u64> {
        self.try_into().map_err(|_| BnConversionError)
    }

    // U192安全转换为u64，返回Option
    #[inline]
    pub fn to_u64(self) -> Option<u64> {
        self.try_to_u64().map_or(None, Some)
    }

    // U192安全转换为u128，返回Result
    #[inline]
    pub fn try_to_u128(self) -> ClearingHouseResult<u128> {
        self.try_into().map_err(|_| BnConversionError)
    }

    // U192安全转换为u128，返回Option
    #[inline]
    pub fn to_u128(self) -> Option<u128> {
        self.try_to_u128().map_or(None, Some)
    }
}

impl_anchor_serialize_for_bn!(U192);
impl_anchor_deserialize_for_bn!(U192);

// 在编译时生成高精度无符号大整数类型U256
// 256位的无符号整数，参数4表示底层用4个u64存储数据，即64*4=256 bit
construct_uint! {
    pub struct U256(4);
}

impl U256 {
    // U256安全转换为u64，返回Result
    #[inline]
    pub fn try_to_u64(self) -> ClearingHouseResult<u64> {
        self.try_into().map_err(|_| BnConversionError)
    }

    // U256安全转换为u64，返回Option
    #[inline]
    pub fn to_u64(self) -> Option<u64> {
        self.try_to_u64().map_or(None, Some)
    }

    // U256安全转换为u128，返回Result
    #[inline]
    pub fn try_to_u128(self) -> ClearingHouseResult<u128> {
        self.try_into().map_err(|_| BnConversionError)
    }

    // U256安全转换为u128，返回Option
    #[inline]
    pub fn to_u128(self) -> Option<u128> {
        self.try_to_u128().map_or(None, Some)
    }
}

impl_anchor_serialize_for_bn!(U256);
impl_anchor_deserialize_for_bn!(U256);
