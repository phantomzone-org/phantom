#[cfg(test)]
mod tests {
    use crate::{PC_UPDATE, RAM_UPDATE, RD_UPDATE};

    use super::*;

    #[test]
    fn add() {
        test_instruction(
            0,
            0,
            0b0110011,
            (RD_UPDATE::ADD, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn and() {
        test_instruction(
            0,
            0b111,
            0b0110011,
            (RD_UPDATE::AND, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn or() {
        test_instruction(
            0,
            0b110,
            0b0110011,
            (RD_UPDATE::OR, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn sll() {
        test_instruction(
            0,
            0b001,
            0b0110011,
            (RD_UPDATE::SLL, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn slt() {
        test_instruction(
            0,
            0b010,
            0b0110011,
            (RD_UPDATE::SLT, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn sltu() {
        test_instruction(
            0,
            0b011,
            0b0110011,
            (RD_UPDATE::SLTU, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn sra() {
        test_instruction(
            0b0100000,
            0b101,
            0b0110011,
            (RD_UPDATE::SRA, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn srl() {
        test_instruction(
            0,
            0b101,
            0b0110011,
            (RD_UPDATE::SRL, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn sub() {
        test_instruction(
            0b0100000,
            0,
            0b0110011,
            (RD_UPDATE::SUB, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn xor() {
        test_instruction(
            0,
            0b100,
            0b0110011,
            (RD_UPDATE::XOR, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn mul() {
        test_instruction(
            0b0000001,
            0b000,
            0b0110011,
            (RD_UPDATE::MUL, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn mulh() {
        test_instruction(
            0b0000001,
            0b001,
            0b0110011,
            (RD_UPDATE::MULH, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn mulhsu() {
        test_instruction(
            0b0000001,
            0b010,
            0b0110011,
            (RD_UPDATE::MULHSU, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn mulhu() {
        test_instruction(
            0b0000001,
            0b011,
            0b0110011,
            (RD_UPDATE::MULHU, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn div() {
        test_instruction(
            0b0000001,
            0b100,
            0b0110011,
            (RD_UPDATE::DIV, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn divu() {
        test_instruction(
            0b0000001,
            0b101,
            0b0110011,
            (RD_UPDATE::DIVU, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn rem() {
        test_instruction(
            0b0000001,
            0b110,
            0b0110011,
            (RD_UPDATE::REM, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn remu() {
        test_instruction(
            0b0000001,
            0b111,
            0b0110011,
            (RD_UPDATE::REMU, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }
}

use crate::instructions::{Instruction, InstructionsParser};
use crate::{PC_UPDATE, RAM_UPDATE, RD_UPDATE};
#[allow(dead_code)]
fn test_instruction(
    funct7: u32,
    funct3: u32,
    op_code: u32,
    opid: (RD_UPDATE, RAM_UPDATE, PC_UPDATE),
) {
    // 00000 | 00 | rs2[24:20] | rs1[19:15] | funct3 | rd[11:7] |
    let imm: u32 = 0;
    let rs2: u32 = 0b11011;
    let rs1: u32 = 0b10011;
    let rd: u32 = 0b01011;
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
