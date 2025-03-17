//! xori
//!
//! # Format
//!
//! xori rd,rs1,imm
//!
//! # Description
//!
//! Performs bitwise XOR on register rs1 and the sign-extended 12-bit
//! immediate and place the result in rd.
//! Note, “XORI rd, rs1, -1” performs a bitwise logical inversion of
//! register rs1(assembler pseudo-instruction NOT rd, rs)
//!
//! # Implementation
//!
//! x[rd] = x[rs1] ^ sext(immediate)

use crate::instructions::{decompose, reconstruct};

pub struct Xori();

impl Xori {
    pub fn apply(imm: &[u8; 8], x_rs1: &[u8; 8], _x_rs2: &[u8; 8]) -> [u8; 8] {
        let imm_u32: u32 = reconstruct(imm);
        let x_rs1_u32: u32 = reconstruct(x_rs1);
        decompose(x_rs1_u32 ^ imm_u32)
    }
}
