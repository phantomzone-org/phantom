//! addi: add immediate
//!
//! # Description
//!
//! Adds the sign-extended 12-bit immediate to register rs1.
//! Arithmetic overflow is ignored and the result is simply the
//! low XLEN bits of the result. ADDI rd, rs1, 0 is used to
//! implement the MV rd, rs1 assembler pseudo-instruction.
//!
//! # Implementation
//!
//! | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] + sext(imm[11:0])

use crate::instructions::{decompose, reconstruct, Arithmetic};

pub struct Addi();

impl Arithmetic for Addi {
    fn apply(imm: &[u8; 8], x_rs1: &[u8; 8], _x_rs2: &[u8; 8]) -> [u8; 8] {
        let imm_u32: u32 = reconstruct(imm);
        let x_rs1_u32: u32 = reconstruct(x_rs1);
        decompose(x_rs1_u32.wrapping_add(imm_u32))
    }
}
