pub const IMMMASK20: u32 = 0x0010_0000;
pub const IMMMASK19: u32 = 0x000F_F000;
pub const IMMMASK11: u32 = 0x0000_0800;
pub const IMMMASK10: u32 = 0x0000_07FE;
pub const OPMASK: u32 = 0x0000_0FFF;
pub const IMMSEXTMASK: u32 = 0xFFF0_0000;

pub const IMMSHIFT20: u32 = 11;
pub const IMMSHIFT19: u32 = 0;
pub const IMMSHIFT11: u32 = 9;
pub const IMMSHIFT10: u32 = 20;

#[inline(always)]
pub fn set_immediate(instruction: &mut u32, immediate: u32) {
    *instruction = (*instruction & OPMASK)
        | (immediate & IMMMASK20) << IMMSHIFT20
        | (immediate & IMMMASK19) << IMMSHIFT19
        | (immediate & IMMMASK11) << IMMSHIFT11
        | (immediate & IMMMASK10) << IMMSHIFT10
}

#[inline(always)]
pub fn get_immediate(instruction: &u32) -> u32 {
    (instruction >> IMMSHIFT20) & IMMMASK20
        | (instruction >> IMMSHIFT19) & IMMMASK19
        | (instruction >> IMMSHIFT11) & IMMMASK11
        | (instruction >> IMMSHIFT10) & IMMMASK10
        | ((instruction >> 31) & 1) * IMMSEXTMASK
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{instructions::sext, PC_UPDATE, RD_UPDATE, RAM_UPDATE};
    #[test]
    fn imm_encoding() {
        (1..21).for_each(|i| {
            let immediate: u32 = 1 << i;
            let mut instruction: u32 = 0;
            set_immediate(&mut instruction, immediate);
            assert_eq!(sext(immediate, 20), get_immediate(&instruction));
        })
    }

    #[test]
    fn jal() {
        test_instruction(0b1101111, (RD_UPDATE::JAL, RAM_UPDATE::NONE, PC_UPDATE::JAL))
    }
}

use crate::instructions::{sext, Instruction, InstructionsParser};
use crate::{PC_UPDATE, RD_UPDATE, RAM_UPDATE};
#[allow(dead_code)]
fn test_instruction(op_code: u32, op_id: (RD_UPDATE, RAM_UPDATE, PC_UPDATE)) {
    let imm: u32 = 0xABCDE << 1;
    let rs2: u32 = 0;
    let rs1: u32 = 0;
    let rd: u32 = 0b01011;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.set_immediate(imm);
    instruction.set_rd(rd);
    instruction.print();
    let mut m: InstructionsParser = InstructionsParser::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(
        0,
        sext(imm, 20) as i64,
        rs2 as i64,
        rs1 as i64,
        rd as i64,
        op_id.0 as i64,
        op_id.1 as i64,
        op_id.2 as i64,
    );
}
