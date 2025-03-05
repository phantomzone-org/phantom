pub mod addi;
pub mod andi;
pub mod jalr;
pub mod ori;
pub mod slli;
pub mod slti;
pub mod sltiu;
pub mod srai;
pub mod srli;
pub mod xori;

pub const IMMSHIFT: u32 = 20;
pub const OPMASK: u32 = 0x000F_FFFF;
pub const IMMSEXTMASK: u32 = 0xFFFF_F000;

#[inline(always)]
pub fn set_immediate(instruction: &mut u32, immediate: u32) {
    *instruction = (*instruction & OPMASK) | (immediate << IMMSHIFT);
}

#[inline(always)]
pub fn get_immediate(instruction: &u32) -> u32 {
    (instruction >> IMMSHIFT) | ((instruction >> 31) & 1) * IMMSEXTMASK
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::instructions::{sext, OpID};

    #[test]
    fn imm_encoding() {
        (0..12).for_each(|i| {
            let immediate: u32 = 1 << i;
            let mut instruction: u32 = 0;
            set_immediate(&mut instruction, immediate);
            assert_eq!(sext(immediate, 11), get_immediate(&instruction));
        })
    }

    #[test]
    fn addi() {
        test_instruction(0b000, 0b0010011, OpID::ADDI)
    }

    #[test]
    fn slti() {
        test_instruction(0b010, 0b0010011, OpID::SLTI)
    }

    #[test]
    fn sltiu() {
        test_instruction(0b011, 0b0010011, OpID::SLTIU)
    }

    #[test]
    fn xori() {
        test_instruction(0b100, 0b0010011, OpID::XORI)
    }

    #[test]
    fn ori() {
        test_instruction(0b110, 0b0010011, OpID::ORI)
    }

    #[test]
    fn andi() {
        test_instruction(0b111, 0b0010011, OpID::ANDI)
    }

    #[test]
    fn slli() {
        test_instruction_shamt(0b000000011111, 0b001, OpID::SLLI)
    }

    #[test]
    fn srli() {
        test_instruction_shamt(0b000000011111, 0b101, OpID::SRLI)
    }

    #[test]
    fn srai() {
        test_instruction_shamt(0b010000011111, 0b101, OpID::SRAI)
    }

    #[test]
    fn jalr() {
        test_instruction(0b000, 0b1100111, OpID::JALR)
    }

    #[test]
    fn lb() {
        test_instruction(0, 0b0000011, OpID::LB)
    }

    #[test]
    fn lh() {
        test_instruction(0b001, 0b0000011, OpID::LH)
    }

    #[test]
    fn lw() {
        test_instruction(0b010, 0b0000011, OpID::LW)
    }

    #[test]
    fn lbu() {
        test_instruction(0b100, 0b0000011, OpID::LBU)
    }

    #[test]
    fn lhu() {
        test_instruction(0b101, 0b0000011, OpID::LHU)
    }
}

use crate::instructions::{decompose, sext, Instruction, Instructions};
#[allow(dead_code)]
fn test_instruction(funct3: u8, op_code: u8, opid: (u8, u8, u8)) {
    // imm[31:20] | rs1[19:15] | funct3 | rd[11:7] | op_code
    // imm[11: 0]
    let funct3: u8 = funct3;
    let imm: u32 = 0xABC;
    let rs2: u8 = 0;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.set_immediate(imm);
    instruction.set_funct3(funct3);
    instruction.set_rs1(rs1);
    instruction.set_rd(rd);
    let mut m: Instructions = Instructions::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(
        0,
        decompose(sext(imm, 11)),
        rs2,
        rs1,
        rd,
        opid.0,
        opid.1,
        opid.2,
    );
}

#[allow(dead_code)]
fn test_instruction_shamt(imm: u32, funct3: u8, opid: (u8, u8, u8)) {
    // 0000000 | shamt[24:20] | rs1[19:15] | funct3 | rd[11:7] | 0010011
    let op_code: u8 = 0b0010011;
    let funct3: u8 = funct3;
    let rs2: u8 = 0;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.set_immediate(imm);
    instruction.set_funct3(funct3);
    instruction.set_rs1(rs1);
    instruction.set_rd(rd);
    let mut m: Instructions = Instructions::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(
        0,
        decompose(imm & 0x1F),
        rs2,
        rs1,
        rd,
        opid.0,
        opid.1,
        opid.2,
    );
}
