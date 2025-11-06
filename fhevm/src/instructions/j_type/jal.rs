//! JAL
//!
//! # Format
//!
//! jal rd,offset
//!
//! # Description
//!
//! Jump to address and place return address in rd.
//!
//! Implementation
//!
//! jal    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = pc + 4; pc += sext(imm[19:0])

use crate::instructions::{decompose, reconstruct};

pub struct Jal();

impl Jal {
    pub fn apply_pc(
        imm: &[u32; 8],
        _x_rs1: &[u32; 8],
        _x_rs2: &[u32; 8],
        pc: &[u32; 8],
    ) -> [u32; 8] {
        decompose(reconstruct(pc).wrapping_add(reconstruct(imm)))
    }
    pub fn apply_rd(
        _imm: &[u32; 8],
        _x_rs1: &[u32; 8],
        _x_rs2: &[u32; 8],
        pc: &[u32; 8],
    ) -> [u32; 8] {
        decompose(reconstruct(pc) + 4)
    }
}
