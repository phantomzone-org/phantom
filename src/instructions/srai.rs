//! slli
//!
//! # Format
//!
//! srai rd,rs1,shamt
//!
//! # Description
//!
//! Performs arithmetic right shift on the value in register
//! rs1 by the shift amount held in the lower 5 bits of the
//! immediate In RV64, bit-25 is used to shamt[5].
//!
//! # Implementation
//!
//! x[rd] = x[rs1] >>s shamt

#[cfg(test)]
use crate::instructions::{encode_0010011, Instructions};
#[test]
fn instruction_parsing() {
    // 01000 | 00 | imm[24:20] | rs1[19:15] | 101 | rd[11:7] | 00100 | 11
    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = 0;
    let imm_7: u8 = 0b0001;
    let imm_3: u8 = 0b1001;
    let rs2: u8 = 0;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let rd_w: u8 = 11;
    let mem_w: u8 = 0;
    let pc_w: u8 = 0;

    let rv32: u32 = encode_0010011(0b0100, imm_7, imm_3, rs1, 0b101, rd);

    let mut m: Instructions = Instructions::new();
    m.add(rv32);
    m.assert_size(1);
    m.assert_instruction(
        0, imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    );
}
