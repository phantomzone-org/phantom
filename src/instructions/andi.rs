//! ori
//!
//! # Format
//!
//! andi rd,rs1,imm
//!
//! # Description
//!
//! Performs bitwise AND on register rs1 and the sign-
//! extended 12-bit immediate and place the result in rd.
//!
//! # Implementation
//!
//! | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] & sext(imm[11:0])
//!

use super::{decompose, reconstruct, Arithmetic};

pub struct Andi();

impl Arithmetic for Andi {
    fn apply(&self, imm: &[u8; 8], x_rs1: &[u8; 8], _x_rs2: &[u8; 8]) -> [u8; 8] {
        let imm_u32: u32 = reconstruct(imm);
        let x_rs1_u32: u32 = reconstruct(x_rs1);
        decompose(x_rs1_u32 & imm_u32)
    }
}

#[cfg(test)]
use crate::instructions::{test_i_type, OpID};
#[test]
fn instruction_parsing() {
    test_i_type(0b111, 0b0010011, OpID::ANDI)
}
