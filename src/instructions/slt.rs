#[cfg(test)]
use crate::instructions::{encode_0110011, Instructions};
#[test]
fn instruction_parsing() {
    // 00000 | 00 | rs2[24:20] | rs1[19:15] | 010 | rd[11:7] | 01100 | 11
    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = 0;
    let imm_7: u8 = 0;
    let imm_3: u8 = 0;
    let rs2: u8 = 0b11011;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let rd_w: u8 = 15;
    let mem_w: u8 = 0;
    let pc_w: u8 = 0;

    let rv32: u32 = encode_0110011(0, rs2, rs1, 0b010, rd);

    let mut m: Instructions = Instructions::new();
    m.add(rv32);
    m.assert_size(1);
    m.assert_instruction(
        0, imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    );
}
