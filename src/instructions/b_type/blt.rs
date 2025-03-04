//! | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] <s x[rs2]), pc += sext(imm[19:0])

use crate::instructions::{decompose, reconstruct, PcUpdates};

pub struct Blt();

impl PcUpdates for Blt {
    fn apply(
        &self,
        imm: &[u8; 8],
        x_rs1: &[u8; 8],
        x_rs2: &[u8; 8],
        pc: &[u8; 8],
    ) -> ([u8; 8], [u8; 8]) {
        let x_rs1_u32: u32 = reconstruct(x_rs1);
        let x_rs2_u32: u32 = reconstruct(x_rs2);
        if (x_rs1_u32 as i32) < (x_rs2_u32 as i32) {
            (
                [0u8; 8],
                decompose(reconstruct(pc).wrapping_add(reconstruct(imm))),
            )
        } else {
            ([0u8; 8], *pc)
        }
    }
}
