pub const OPMASK: u32 = 0x0000_0FFF;
pub const IMMMASK: u32 = 0xFFFF_F000;

#[inline(always)]
pub fn set_immediate(mut instruction: Instruction, immediate: u32) -> Instruction {
    instruction.0 = (instruction.0 & OPMASK) | ((immediate << 12) & IMMMASK);
    instruction
}

#[inline(always)]
pub fn get_immediate(instruction: &u32) -> u32 {
    (instruction & IMMMASK) >> 12
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PC_UPDATE, RAM_UPDATE, RD_UPDATE};

    #[test]
    fn imm_encoding() {
        (12..32).for_each(|i| {
            let immediate: u32 = 1 << i;
            let mut instruction: u32 = 0;
            instruction = set_immediate(Instruction(instruction), immediate).0;
            assert_eq!(immediate, get_immediate(&instruction));
        })
    }

    #[test]
    fn lui() {
        test_instruction(
            0b0110111,
            (RD_UPDATE::LUI, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }

    #[test]
    fn auipc() {
        test_instruction(
            0b0010111,
            (RD_UPDATE::AUIPC, RAM_UPDATE::NONE, PC_UPDATE::NONE),
        )
    }
}

use crate::{
    instructions::{Instruction, InstructionsParser},
    PC_UPDATE, RAM_UPDATE, RD_UPDATE,
};
#[allow(dead_code)]
fn test_instruction(op_code: u32, op_id: (RD_UPDATE, RAM_UPDATE, PC_UPDATE)) {
    let imm: u32 = 0xABCD_E000;
    let rs2: u32 = 0;
    let rs1: u32 = 0;
    let rd: u32 = 0b01011;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction = instruction.set_imm(imm);
    instruction = instruction.set_rd(rd);
    let mut m: InstructionsParser = InstructionsParser::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(
        0,
        imm as i64,
        rs2 as i64,
        rs1 as i64,
        rd as i64,
        op_id.0 as i64,
        op_id.1 as i64,
        op_id.2 as i64,
    );
}
