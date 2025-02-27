//! | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] == x[rs2]), pc += sext(imm[19:0])

use super::{decomp, reconstruct, sext, PcUpdates};
use crate::parameters::{U20DECOMP, U32DECOMP};

pub struct Beq();

impl PcUpdates for Beq {
    fn apply(&self, imm: &[u32], x_rs1: &[u32], x_rs2: &[u32], pc: &[u32]) -> (Vec<u32>, Vec<u32>) {
        let x_rs1_u32: u32 = reconstruct(x_rs1, &U32DECOMP);
        let x_rs2_u32: u32 = reconstruct(x_rs2, &U32DECOMP);
        if x_rs1_u32 == x_rs2_u32 {
            (
                Vec::new(),
                decomp(
                    reconstruct(pc, &U32DECOMP)
                        .wrapping_add(sext(reconstruct(imm, &U20DECOMP), 20)),
                    &U32DECOMP,
                ),
            )
        } else {
            (Vec::new(), pc.into())
        }
    }
}

#[cfg(test)]
use crate::instructions::{encode_1100011, Instructions};
#[test]
fn instruction_parsing() {
    // imm[12|10:5] | rs2[24:20] | rs1[19:15] | 000 | imm[4:1|11] | 11000 | 11

    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = 0b1011; // 12 11 10  9
    let imm_7: u8 = 0b0111; //  8  7  6  5
    let imm_3: u8 = 0b1001; //  4  3  2  1
    let rs2: u8 = 0b01101;
    let rs1: u8 = 0b01001;
    let rd: u8 = 0;
    let rd_w: u8 = 0;
    let mem_w: u8 = 0;
    let pc_w: u8 = 3;

    let rv32: u32 = encode_1100011(imm_11, imm_7, imm_3, rs2, rs1, 0b000);

    assert_eq!(rv32, 0xeed48963);

    let mut m: Instructions = Instructions::new();
    m.add(rv32);
    m.assert_size(1);
    m.assert_instruction(
        0, imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    );
}
