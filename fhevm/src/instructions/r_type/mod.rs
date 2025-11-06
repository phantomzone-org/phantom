pub mod add;
pub mod and;
pub mod div;
pub mod divu;
pub mod mul;
pub mod mulh;
pub mod mulhsu;
pub mod mulhu;
pub mod or;
pub mod rem;
pub mod remu;
pub mod sll;
pub mod slt;
pub mod sltu;
pub mod sra;
pub mod srl;
pub mod sub;
pub mod xor;

#[cfg(test)]
mod tests {
    use crate::{OpIDPCUpdate, OpIDRd, OpIDStore};

    use super::*;

    #[test]
    fn add() {
        test_instruction(
            0,
            0,
            0b0110011,
            (OpIDRd::ADD, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn and() {
        test_instruction(
            0,
            0b111,
            0b0110011,
            (OpIDRd::AND, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn or() {
        test_instruction(
            0,
            0b110,
            0b0110011,
            (OpIDRd::OR, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn sll() {
        test_instruction(
            0,
            0b001,
            0b0110011,
            (OpIDRd::SLL, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn slt() {
        test_instruction(
            0,
            0b010,
            0b0110011,
            (OpIDRd::SLT, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn sltu() {
        test_instruction(
            0,
            0b011,
            0b0110011,
            (OpIDRd::SLTU, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn sra() {
        test_instruction(
            0b0100000,
            0b101,
            0b0110011,
            (OpIDRd::SRA, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn srl() {
        test_instruction(
            0,
            0b101,
            0b0110011,
            (OpIDRd::SRL, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn sub() {
        test_instruction(
            0b0100000,
            0,
            0b0110011,
            (OpIDRd::SUB, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn xor() {
        test_instruction(
            0,
            0b100,
            0b0110011,
            (OpIDRd::XOR, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn mul() {
        test_instruction(
            0b0000001,
            0b000,
            0b0110011,
            (OpIDRd::MUL, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn mulh() {
        test_instruction(
            0b0000001,
            0b001,
            0b0110011,
            (OpIDRd::MULH, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn mulhsu() {
        test_instruction(
            0b0000001,
            0b010,
            0b0110011,
            (OpIDRd::MULHSU, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn mulhu() {
        test_instruction(
            0b0000001,
            0b011,
            0b0110011,
            (OpIDRd::MULHU, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn div() {
        test_instruction(
            0b0000001,
            0b100,
            0b0110011,
            (OpIDRd::DIV, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn divu() {
        test_instruction(
            0b0000001,
            0b101,
            0b0110011,
            (OpIDRd::DIVU, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn rem() {
        test_instruction(
            0b0000001,
            0b110,
            0b0110011,
            (OpIDRd::REM, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn remu() {
        test_instruction(
            0b0000001,
            0b111,
            0b0110011,
            (OpIDRd::REMU, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }
}

use crate::instructions::{Instruction, InstructionsParser};
#[allow(dead_code)]
fn test_instruction(funct7: u32, funct3: u32, op_code: u32, opid: (u32, u32, u32)) {
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
