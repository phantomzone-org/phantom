//! sltiu: set less than immediate
//!
//! # Format
//!
//! sltiu rd,rs1,imm
//!
//! # Description
//!
//! Place the value 1 in register rd if register rs1 is
//! less than the immediate when both are treated as
//! unsigned numbers, else 0 is written to rd.
//!
//! # Implementation
//!
//! x[rd] = x[rs1] <u sext(immediate)

#[cfg(test)]
use crate::instructions::{encode_0010011, Instructions};
#[test]
fn instruction_parsing() {
    // imm[31:20] | rs1[19:15] | 011 | rd[11:7] | 00100 | 11
    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = 0b1010;
    let imm_7: u8 = 0b1001;
    let imm_3: u8 = 0b1000;
    let rs2: u8 = 0;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let rd_w: u8 = 5;
    let mem_w: u8 = 0;
    let pc_w: u8 = 0;

    let rv32: u32 = encode_0010011(imm_11, imm_7, imm_3, rs1, 0b011, rd);

    let mut m: Instructions = Instructions::new();
    m.add(rv32);
    m.assert_size(1);
    m.assert_instruction(
        0, imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    );
}
