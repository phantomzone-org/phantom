//! LUI: load upper immediate.
//!
//! # Format
//!
//! lui rd,imm
//!
//! # Description
//!
//! Build 32-bit constants and uses the U-type format.
//! LUI places the U-immediate value in the top 20 bits
//! of the destination register rd, filling in the lowest
//! 12 bits with zeros.
//!
//! # Implementation
//!
//! | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = sext(imm[19:0] << 12)
//!

use super::{decompose, reconstruct, Arithmetic};

pub struct Lui();

impl Arithmetic for Lui {
    fn apply(&self, imm: &[u8; 8], _x_rs1: &[u8; 8], _x_rs2: &[u8; 8]) -> [u8; 8] {
        decompose(reconstruct(imm))
    }
}

#[cfg(test)]
use crate::instructions::{test_u_type, OpID};
#[test]
fn instruction_parsing() {
    test_u_type(0b01101111, OpID::LUI)
}
