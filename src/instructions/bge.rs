//! bge    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] >=s x[rs2]), pc += sext(imm[19:0])

use super::{decompose, reconstruct, PcUpdates};

pub struct Bge();

impl PcUpdates for Bge {
    fn apply(
        &self,
        imm: &[u8; 8],
        x_rs1: &[u8; 8],
        x_rs2: &[u8; 8],
        pc: &[u8; 8],
    ) -> ([u8; 8], [u8; 8]) {
        let x_rs1_u32: u32 = reconstruct(x_rs1);
        let x_rs2_u32: u32 = reconstruct(x_rs2);
        if x_rs1_u32 as i32 >= x_rs2_u32 as i32 {
            (
                [0u8; 8],
                decompose(reconstruct(pc).wrapping_add(reconstruct(imm))),
            )
        } else {
            ([0u8; 8], *pc)
        }
    }
}

#[cfg(test)]
use crate::instructions::{test_b_type, OpID};
#[test]
fn instruction_parsing() {
    test_b_type(0b101, 0b1100011, OpID::BEQ)
}
