pub mod add;
pub mod and;
pub mod or;
pub mod sll;
pub mod slt;
pub mod sltu;
pub mod sra;
pub mod srl;
pub mod sub;
pub mod xor;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::OpID;

    #[test]
    fn add() {
        test_instruction(0, 0, 0b0110011, OpID::ADD)
    }

    #[test]
    fn and() {
        test_instruction(0, 0b111, 0b0110011, OpID::AND)
    }

    #[test]
    fn or() {
        test_instruction(0, 0b110, 0b0110011, OpID::OR)
    }

    #[test]
    fn sll() {
        test_instruction(0, 0b001, 0b0110011, OpID::SLL)
    }

    #[test]
    fn slt() {
        test_instruction(0, 0b010, 0b0110011, OpID::SLT)
    }

    #[test]
    fn sltu() {
        test_instruction(0, 0b011, 0b0110011, OpID::SLTU)
    }

    #[test]
    fn sra() {
        test_instruction(0b0100000, 0b101, 0b0110011, OpID::SRA)
    }

    #[test]
    fn srl() {
        test_instruction(0, 0b101, 0b0110011, OpID::SRL)
    }

    #[test]
    fn sub() {
        test_instruction(0b0100000, 0, 0b0110011, OpID::SUB)
    }

    #[test]
    fn xor() {
        test_instruction(0, 0b100, 0b0110011, OpID::XOR)
    }
}

use crate::instructions::{decompose, sext, Instruction, Instructions};
#[allow(dead_code)]
fn test_instruction(funct7: u8, funct3: u8, op_code: u8, opid: (u8, u8, u8)) {
    // 00000 | 00 | rs2[24:20] | rs1[19:15] | funct3 | rd[11:7] |
    let imm: u32 = 0;
    let rs2: u8 = 0b11011;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.set_funct3(funct3);
    instruction.set_funct7(funct7);
    instruction.set_rs2(rs2);
    instruction.set_rs1(rs1);
    instruction.set_rd(rd);
    let mut m: Instructions = Instructions::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(0, decompose(imm), rs2, rs1, rd, opid.0, opid.1, opid.2);
}
