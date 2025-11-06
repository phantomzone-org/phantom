//! ori
//!
//! # Format
//!
//! ori rd,rs1,imm
//!
//! # Description
//!
//! Performs bitwise OR on register rs1 and the
//! sign-extended 12-bit immediate and place the result in rd
//!
//! # Implementation
//!
//! | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] | sext(imm[11:0])

use crate::instructions::{decompose, reconstruct};

pub struct Ori();

impl Ori {
    pub fn apply(imm: &[u32; 8], x_rs1: &[u32; 8], _x_rs2: &[u32; 8]) -> [u32; 8] {
        let imm_u32: u32 = reconstruct(imm);
        let x_rs1_u32: u32 = reconstruct(x_rs1);
        decompose(x_rs1_u32 | imm_u32)
    }
}
