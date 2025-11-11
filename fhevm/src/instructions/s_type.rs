pub const IMMMASK11: u32 = 0x0000_0FE0;
pub const IMMMASK4: u32 = 0x0000_001F;
pub const OPMASK: u32 = 0x01FF_F07F;
pub const IMMSHIFT11: u32 = 20;
pub const IMMSHIFT4: u32 = 7;
pub const IMMSEXTMASK: u32 = 0xFFFF_F000;

#[inline(always)]
pub fn set_immediate(mut instruction: Instruction, immediate: u32) -> Instruction {
    instruction.0 = (instruction.0 & OPMASK)
        | (immediate & IMMMASK11) << IMMSHIFT11
        | (immediate & IMMMASK4) << IMMSHIFT4;
    instruction
}

#[inline(always)]
pub fn get_immediate(instruction: &u32) -> u32 {
    (instruction >> IMMSHIFT11) & IMMMASK11
        | (instruction >> IMMSHIFT4) & IMMMASK4
        | ((instruction >> 31) & 1) * IMMSEXTMASK
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::{sext, PC_UPDATE, RAM_UPDATE, RD_UPDATE};

    #[test]
    fn imm_encoding() {
        (0..12).for_each(|i| {
            let immediate: u32 = 1 << i;
            let mut instruction: u32 = 0;
            instruction = set_immediate(Instruction(instruction), immediate).0;
            assert_eq!(sext(immediate, 11), get_immediate(&instruction));
        })
    }

    #[test]
    fn sb() {
        test_instruction(
            0b000,
            0b0100011,
            (RD_UPDATE::NONE, RAM_UPDATE::SB, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn sh() {
        test_instruction(
            0b001,
            0b0100011,
            (RD_UPDATE::NONE, RAM_UPDATE::SH, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn sw() {
        test_instruction(
            0b010,
            0b0100011,
            (RD_UPDATE::NONE, RAM_UPDATE::SW, PC_UPDATE::NONE),
        )
    }
}

use crate::instructions::{
    sext, Instruction, InstructionsParser, PC_UPDATE, RAM_UPDATE, RD_UPDATE,
};
#[allow(dead_code)]
fn test_instruction(funct3: u32, op_code: u32, op_id: (RD_UPDATE, RAM_UPDATE, PC_UPDATE)) {
    // imm[11:5] | rs2[24:20] | rs1[19:15] | 000 | imm[4:0] | 0100011
    let imm: u32 = 0xABC;
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
        sext(imm, 11) as i64,
        rs2 as i64,
        rs1 as i64,
        rd as i64,
        op_id.0 as i64,
        op_id.1 as i64,
        op_id.2 as i64,
    );
}
