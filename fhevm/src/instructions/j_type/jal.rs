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

use crate::instructions::{decompose, reconstruct, PcUpdates};

pub struct Jal();

impl PcUpdates for Jal {
    fn apply(
        imm: &[u8; 8],
        _x_rs1: &[u8; 8],
        _x_rs2: &[u8; 8],
        pc: &[u8; 8],
    ) -> ([u8; 8], [u8; 8]) {
        (
            decompose(reconstruct(pc) + 4),
            decompose(reconstruct(pc).wrapping_add(reconstruct(imm))),
        )
    }
}
