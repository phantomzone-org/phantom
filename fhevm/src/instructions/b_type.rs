pub const IMMMASK12: u32 = 0x0000_0800;
pub const IMMMASK11: u32 = 0x0000_0400;
pub const IMMMASK10: u32 = 0x0000_03F0;
pub const IMMMASK4: u32 = 0x0000_000F;
pub const OPMASK: u32 = 0x01FF_F07F;
pub const IMMSEXTMASK: u32 = 0xFFFF_F800;

pub const IMMSHIFT12: u32 = 20;
pub const IMMSHIFT11: u32 = 3;
pub const IMMSHIFT10: u32 = 21;
pub const IMMSHIFT4: u32 = 8;

#[inline(always)]
pub fn set_immediate(mut instruction: Instruction, immediate: u32) -> Instruction {
    let imm_shift: u32 = immediate >> 1;
    instruction.0 = (instruction.0 & OPMASK)
        | (imm_shift & IMMMASK12) << IMMSHIFT12
        | (imm_shift & IMMMASK11) >> IMMSHIFT11
        | (imm_shift & IMMMASK10) << IMMSHIFT10
        | (imm_shift & IMMMASK4) << IMMSHIFT4;
    instruction
}

#[inline(always)]
pub fn get_immediate(instruction: &u32) -> u32 {
    ((instruction >> IMMSHIFT12) & IMMMASK12
        | (instruction << IMMSHIFT11) & IMMMASK11
        | (instruction >> IMMSHIFT10) & IMMMASK10
        | (instruction >> IMMSHIFT4) & IMMMASK4
        | ((instruction >> 31) & 1) * IMMSEXTMASK)
        << 1
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{instructions::sext, PC_UPDATE, RAM_UPDATE, RD_UPDATE};

    #[test]
    fn imm_encoding() {
        (1..13).for_each(|i| {
            let immediate: u32 = 1 << i;
            let mut instruction: u32 = 0;
            instruction = set_immediate(Instruction(instruction), immediate).0;
            assert_eq!(sext(immediate, 12), get_immediate(&instruction));
        })
    }

    #[test]
    fn beq() {
        test_instruction(
            0b000,
            0b1100011,
            (RD_UPDATE::NONE, RAM_UPDATE::NONE, PC_UPDATE::BEQ),
        )
    }

    #[test]
    fn bge() {
        test_instruction(
            0b101,
            0b1100011,
            (RD_UPDATE::NONE, RAM_UPDATE::NONE, PC_UPDATE::BGE),
        )
    }

    #[test]
    fn bgeu() {
        test_instruction(
            0b111,
            0b1100011,
            (RD_UPDATE::NONE, RAM_UPDATE::NONE, PC_UPDATE::BGEU),
        )
    }

    #[test]
    fn blt() {
        test_instruction(
            0b100,
            0b1100011,
            (RD_UPDATE::NONE, RAM_UPDATE::NONE, PC_UPDATE::BLT),
        )
    }

    #[test]
    fn bltu() {
        test_instruction(
            0b110,
            0b1100011,
            (RD_UPDATE::NONE, RAM_UPDATE::NONE, PC_UPDATE::BLTU),
        )
    }

    #[test]
    fn bne() {
        test_instruction(
            0b001,
            0b1100011,
            (RD_UPDATE::NONE, RAM_UPDATE::NONE, PC_UPDATE::BNE),
        )
    }
}

use crate::instructions::{sext, Instruction, InstructionsParser};
use crate::{PC_UPDATE, RAM_UPDATE, RD_UPDATE};
#[allow(dead_code)]
fn test_instruction(funct3: u32, op_code: u32, op_id: (RD_UPDATE, RAM_UPDATE, PC_UPDATE)) {
    let imm: u32 = 0xABC << 1;
    let rs2: u32 = 0b11011;
    let rs1: u32 = 0b10011;
    let rd: u32 = 0;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction = instruction.set_imm(imm);
    instruction = instruction.set_funct3(funct3);
    instruction = instruction.set_rs2(rs2);
    instruction = instruction.set_rs1(rs1);
    let mut m: InstructionsParser = InstructionsParser::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(
        0,
        sext(imm, 12) as i64,
        rs2 as i64,
        rs1 as i64,
        rd as i64,
        op_id.0 as i64,
        op_id.1 as i64,
        op_id.2 as i64,
    );
}
