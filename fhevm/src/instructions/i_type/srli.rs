//! slli
//!
//! # Format
//!
//! srli rd,rs1,shamt
//!
//! # Description
//!
//! Performs logical right shift on the value in register
//! rs1 by the shift amount held in the lower 5 bits of the
//! immediate In RV64, bit-25 is used to shamt[5].
//!
//! # Implementation
//!
//! x[rd] = x[rs1] >>u shamt

use crate::instructions::{decompose, reconstruct};

pub struct Srli();

impl Srli {
    pub fn apply(imm: &[u8; 8], x_rs1: &[u8; 8], _x_rs2: &[u8; 8]) -> [u8; 8] {
        let imm_u32: u32 = reconstruct(imm);
        let x_rs1_u32: u32 = reconstruct(x_rs1);
        decompose(x_rs1_u32 >> std::cmp::min(imm_u32, u32::BITS - 1))
    }
}
