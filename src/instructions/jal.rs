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

use super::{decomp, reconstruct, sext, PcUpdates};
use crate::parameters::{U20DECOMP, U32DECOMP};

pub struct Jal();

impl PcUpdates for Jal {
    fn apply(
        &self,
        imm: &[u32],
        _x_rs1: &[u32],
        _x_rs2: &[u32],
        pc: &[u32],
    ) -> (Vec<u32>, Vec<u32>) {
        (
            decomp(reconstruct(pc, &U32DECOMP) + 4, &U32DECOMP),
            decomp(
                reconstruct(pc, &U32DECOMP).wrapping_add(sext(reconstruct(imm, &U20DECOMP), 20)),
                &U32DECOMP,
            ),
        )
    }
}
