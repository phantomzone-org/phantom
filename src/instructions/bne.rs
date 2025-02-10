//! bne    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] !=  x[rs2]), pc += sext(imm[19:0])

use super::{decomp, reconstruct, sext, PcUpdates};
use crate::parameters::{U20DECOMP, U32DECOMP};

pub struct Bne();

impl PcUpdates for Bne {
    fn apply(&self, imm: &[u32], x_rs1: &[u32], x_rs2: &[u32], pc: &[u32]) -> (Vec<u32>, Vec<u32>) {
        let x_rs1_u32: u32 = reconstruct(x_rs1, &U32DECOMP);
        let x_rs2_u32: u32 = reconstruct(x_rs2, &U32DECOMP);
        if x_rs1_u32 != x_rs2_u32 {
            (
                Vec::new(),
                decomp(
                    reconstruct(pc, &U32DECOMP)
                        .wrapping_add(sext(reconstruct(imm, &U20DECOMP), 20)),
                    &U32DECOMP,
                ),
            )
        } else {
            (Vec::new(), Vec::new())
        }
    }
}
