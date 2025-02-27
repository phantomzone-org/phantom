//! | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] & x[rs2]

use super::{decomp, reconstruct, Arithmetic};
use crate::parameters::U32DECOMP;

pub struct And();

impl Arithmetic for And {
    fn apply(&self, _imm: &[u32], x_rs1: &[u32], x_rs2: &[u32]) -> Vec<u32> {
        let x_rs1_u32: u32 = reconstruct(x_rs1, &U32DECOMP);
        let x_rs2_u32: u32 = reconstruct(x_rs2, &U32DECOMP);
        decomp(x_rs1_u32 & x_rs2_u32, &U32DECOMP)
    }
}

#[cfg(test)]
use crate::instructions::{encode_0110011, Instructions};
#[test]
fn instruction_parsing() {
    // 00000 | 00 | rs2[24:20] | rs1[19:15] | 111 | rd[11:7] | 01100 | 11
    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = 0;
    let imm_7: u8 = 0;
    let imm_3: u8 = 0;
    let rs2: u8 = 0b11011;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let rd_w: u8 = 21;
    let mem_w: u8 = 0;
    let pc_w: u8 = 0;

    let rv32: u32 = encode_0110011(0, rs2, rs1, 0b111, rd);

    let mut m: Instructions = Instructions::new();
    m.add(rv32);
    m.assert_size(1);
    m.assert_instruction(
        0, imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    );
}
