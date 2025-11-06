//! slli
//!
//! # Format
//!
//! slli rd,rs1,shamt
//!
//! # Description
//!
//! Performs logical left shift on the value in register
//! rs1 by the shift amount held in the lower 5 bits of
//! the immediate In RV64, bit-25 is used to shamt[5].
//!
//! # Implementation
//!
//! x[rd] = x[rs1] << shamt

use crate::instructions::{decompose, reconstruct};

pub struct Slli();

impl Slli {
    pub fn apply(imm: &[u32; 8], x_rs1: &[u32; 8], _x_rs2: &[u32; 8]) -> [u32; 8] {
        let imm_u32: u32 = reconstruct(imm);
        let x_rs1_u32: u32 = reconstruct(x_rs1);
        decompose(x_rs1_u32 << std::cmp::min(imm_u32, u32::BITS - 1))
    }
}
