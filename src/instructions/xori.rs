//! xori
//!
//! # Format
//!
//! xori rd,rs1,imm
//!
//! # Description
//!
//! Performs bitwise XOR on register rs1 and the sign-extended 12-bit
//! immediate and place the result in rd.
//! Note, “XORI rd, rs1, -1” performs a bitwise logical inversion of
//! register rs1(assembler pseudo-instruction NOT rd, rs)
//!
//! # Implementation
//!
//! x[rd] = x[rs1] ^ sext(immediate)

#[cfg(test)]
use crate::instructions::{encode_0010011, Instructions};
#[test]
fn instruction_parsing() {
    // imm[31:20] | rs1[19:15] | 100 | rd[11:7] | 00100 | 11
    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = 0b1010;
    let imm_7: u8 = 0b1001;
    let imm_3: u8 = 0b1000;
    let rs2: u8 = 0;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let rd_w: u8 = 6;
    let mem_w: u8 = 0;
    let pc_w: u8 = 0;

    let rv32: u32 = encode_0010011(imm_11, imm_7, imm_3, rs1, 0b100, rd);

    let mut m: Instructions = Instructions::new();
    m.add(rv32);
    m.assert_size(1);
    m.assert_instruction(
        0, imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    );
}
