//! JALR
//!
//! # Format
//!
//! jalr rd,rs1,offset
//!
//! # Description
//!
//! Jump to address and place return address in rd.
//!
//! Implementation
//!
//! jalr   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | t = pc + 4; pc = (x[rs1] + sext(imm[11:0])) & ~1; x[rd] = t

use crate::instructions::{decompose, reconstruct, PcUpdates};

pub struct Jalr();

impl PcUpdates for Jalr {
    fn apply(imm: &[u8; 8], x_rs1: &[u8; 8], _x_rs2: &[u8; 8], pc: &[u8; 8]) -> ([u8; 8], [u8; 8]) {
        (
            decompose(reconstruct(pc) + 4),
            decompose(reconstruct(x_rs1).wrapping_add(reconstruct(imm)) & !1),
        )
    }
}
