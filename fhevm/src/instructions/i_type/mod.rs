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
    use crate::{instructions::sext, OpIDPCUpdate, OpIDRd, OpIDStore};

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
        test_instruction(
            0b000,
            0b0010011,
            (OpIDRd::ADDI, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn slti() {
        test_instruction(
            0b010,
            0b0010011,
            (OpIDRd::SLTI, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn sltiu() {
        test_instruction(
            0b011,
            0b0010011,
            (OpIDRd::SLTIU, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn xori() {
        test_instruction(
            0b100,
            0b0010011,
            (OpIDRd::XORI, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn ori() {
        test_instruction(
            0b110,
            0b0010011,
            (OpIDRd::ORI, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn andi() {
        test_instruction(
            0b111,
            0b0010011,
            (OpIDRd::ANDI, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn slli() {
        test_instruction_shamt(
            0b000000011111,
            0b001,
            (OpIDRd::SLLI, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn srli() {
        test_instruction_shamt(
            0b000000011111,
            0b101,
            (OpIDRd::SRLI, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn srai() {
        test_instruction_shamt(
            0b010000011111,
            0b101,
            (OpIDRd::SRAI, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn jalr() {
        test_instruction(
            0b000,
            0b1100111,
            (OpIDRd::JALR, OpIDStore::NONE, OpIDPCUpdate::JALR),
        )
    }

    #[test]
    fn lb() {
        test_instruction(
            0,
            0b0000011,
            (OpIDRd::LB, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn lh() {
        test_instruction(
            0b001,
            0b0000011,
            (OpIDRd::LH, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn lw() {
        test_instruction(
            0b010,
            0b0000011,
            (OpIDRd::LW, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn lbu() {
        test_instruction(
            0b100,
            0b0000011,
            (OpIDRd::LBU, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }

    #[test]
    fn lhu() {
        test_instruction(
            0b101,
            0b0000011,
            (OpIDRd::LHU, OpIDStore::NONE, OpIDPCUpdate::NONE),
        )
    }
}

use crate::instructions::{sext, Instruction, InstructionsParser};
#[allow(dead_code)]
fn test_instruction(funct3: u32, op_code: u32, opid: (u32, u32, u32)) {
    // imm[31:20] | rs1[19:15] | funct3 | rd[11:7] | op_code
    // imm[11: 0]
    let funct3: u32 = funct3;
    let imm: u32 = 0xABC;
    let rs2: u32 = 0;
    let rs1: u32 = 0b10011;
    let rd: u32 = 0b01011;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.set_immediate(imm);
    instruction.set_funct3(funct3);
    instruction.set_rs1(rs1);
    instruction.set_rd(rd);
    let mut m: InstructionsParser = InstructionsParser::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(
        0,
        sext(imm, 11) as i64,
        rs2 as i64,
        rs1 as i64,
        rd as i64,
        opid.0 as i64,
        opid.1 as i64,
        opid.2 as i64,
    );
}

#[allow(dead_code)]
fn test_instruction_shamt(imm: u32, funct3: u32, opid: (u32, u32, u32)) {
    // 0000000 | shamt[24:20] | rs1[19:15] | funct3 | rd[11:7] | 0010011
    let op_code: u32 = 0b0010011;
    let funct3: u32 = funct3;
    let rs2: u32 = 0;
    let rs1: u32 = 0b10011;
    let rd: u32 = 0b01011;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.set_immediate(imm);
    instruction.set_funct3(funct3);
    instruction.set_rs1(rs1);
    instruction.set_rd(rd);
    let mut m: InstructionsParser = InstructionsParser::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(
        0,
        (imm & 0x1F) as i64,
        rs2 as i64,
        rs1 as i64,
        rd as i64,
        opid.0 as i64,
        opid.1 as i64,
        opid.2 as i64,
    );
}
