#[cfg(test)]
use crate::instructions::{Instruction, Instructions, OpID};
#[test]
fn instruction_parsing() {
    // imm[11:5] | rs2[24:20] | rs1[19:15] | 001 | imm[4:0] | 0100011
    let op_code: u8 = 0b0100011;
    let funct3: u8 = 0b001;
    let imm_31: u8 = 0;
    let imm_27: u8 = 0;
    let imm_23: u8 = 0;
    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = 0b1010;
    let imm_7: u8 = 0b1001;
    let imm_3: u8 = 0b1000;
    let rs2: u8 = 0b11011;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0;
    let (rd_w, mem_w, pc_w) = OpID::SH;

    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.encode_immediate((imm_11 as u32) << 8 | (imm_7 as u32) << 4 | imm_3 as u32);
    instruction.encode_funct3(funct3);
    instruction.encode_rs2(rs2);
    instruction.encode_rs1(rs1);

    let mut m: Instructions = Instructions::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(
        0, imm_31, imm_27, imm_23, imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w,
        pc_w,
    );
}
