//! slli
//!
//! # Format
//!
//! srai rd,rs1,shamt
//!
//! # Description
//!
//! Performs arithmetic right shift on the value in register
//! rs1 by the shift amount held in the lower 5 bits of the
//! immediate In RV64, bit-25 is used to shamt[5].
//!
//! # Implementation
//!
//! x[rd] = x[rs1] >>s shamt

use crate::instructions::{decompose, reconstruct};

pub struct Srai();

impl Srai {
    pub fn apply(imm: &[u8; 8], x_rs1: &[u8; 8], _x_rs2: &[u8; 8]) -> [u8; 8] {
        let imm_u32: u32 = reconstruct(imm);
        let x_rs1_u32: u32 = reconstruct(x_rs1);
        decompose(((x_rs1_u32 as i32) >> (imm_u32 as i32)) as u32)
    }
}
