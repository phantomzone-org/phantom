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

use super::{decomp, reconstruct, sext, PcUpdates};
use crate::parameters::{U12DECOMP, U32DECOMP};

pub struct Jalr();

impl PcUpdates for Jalr {
    fn apply(
        &self,
        imm: &[u32],
        x_rs1: &[u32],
        _x_rs2: &[u32],
        pc: &[u32],
    ) -> (Vec<u32>, Vec<u32>) {
        (
            decomp(reconstruct(pc, &U32DECOMP) + 4, &U32DECOMP),
            decomp(
                reconstruct(x_rs1, &U32DECOMP).wrapping_add(sext(reconstruct(imm, &U12DECOMP), 12))
                    & !1,
                &U32DECOMP,
            ),
        )
    }
}
