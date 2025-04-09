use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, AnchorDeserialize, AnchorSerialize)]
#[repr(u8)]
pub enum PositionDirection {
    Long,
    Short,
}

unsafe impl Zeroable for PositionDirection {}
unsafe impl Pod for PositionDirection {}
