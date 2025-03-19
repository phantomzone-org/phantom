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
pub mod mul;
pub mod mulh;
pub mod mulhu;
pub mod mulhsu;
pub mod div;
pub mod divu;
pub mod rem;
pub mod remu;

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

    #[test]
    fn mul() {
        test_instruction(0b0000001, 0b000, 0b0110011, OpID::MUL)
    }

    #[test]
    fn mulh() {
        test_instruction(0b0000001, 0b001, 0b0110011, OpID::MULH)
    }

    #[test]
    fn mulhsu() {
        test_instruction(0b0000001, 0b010, 0b0110011, OpID::MULHSU)
    }

    #[test]
    fn mulhu() {
        test_instruction(0b0000001, 0b011, 0b0110011, OpID::MULHU)
    }

    #[test]
    fn div() {
        test_instruction(0b0000001, 0b100, 0b0110011, OpID::DIV)
    }

    #[test]
    fn divu() {
        test_instruction(0b0000001, 0b101, 0b0110011, OpID::DIVU)
    }

    #[test]
    fn rem() {
        test_instruction(0b0000001, 0b110, 0b0110011, OpID::REM)
    }

    #[test]
    fn remu() {
        test_instruction(0b0000001, 0b111, 0b0110011, OpID::REMU)
    }
}

use crate::instructions::{Instruction, InstructionsParser};
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
    let mut m: InstructionsParser = InstructionsParser::new();
    m.add(instruction);
    println!("{:30b}", m.instructions[0]);
    m.assert_size(1);
    m.assert_instruction(
        0,
        imm as i64,
        rs2 as i64,
        rs1 as i64,
        rd as i64,
        opid.0 as i64,
        opid.1 as i64,
        opid.2 as i64,
    );
}
