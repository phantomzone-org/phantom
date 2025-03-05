pub mod auipc;
pub mod lui;

pub const OPMASK: u32 = 0x0000_0FFF;
pub const IMMMASK: u32 = 0xFFFF_F000;

#[inline(always)]
pub fn set_immediate(instruction: &mut u32, immediate: u32) {
    *instruction = (*instruction & OPMASK) | (immediate & IMMMASK);
}

#[inline(always)]
pub fn get_immediate(instruction: &u32) -> u32 {
    instruction & IMMMASK
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::OpID;

    #[test]
    fn imm_encoding() {
        (12..32).for_each(|i| {
            let immediate: u32 = 1 << i;
            let mut instruction: u32 = 0;
            set_immediate(&mut instruction, immediate);
            assert_eq!(immediate, get_immediate(&instruction));
        })
    }

    #[test]
    fn lui() {
        test_instruction(0b0110111, OpID::LUI)
    }

    #[test]
    fn auipc() {
        test_instruction(0b0010111, OpID::AUIPC)
    }
}

use crate::instructions::{decompose, Instruction, Instructions};
#[allow(dead_code)]
fn test_instruction(op_code: u8, op_id: (u8, u8, u8)) {
    let imm: u32 = 0xABCD_E000;
    let rs2: u8 = 0;
    let rs1: u8 = 0;
    let rd: u8 = 0b01011;
    let mut instruction: Instruction = Instruction::new(op_code as u32);
    instruction.set_immediate(imm);
    instruction.set_rd(rd);
    let mut m: Instructions = Instructions::new();
    m.add(instruction);
    m.assert_size(1);
    m.assert_instruction(0, decompose(imm), rs2, rs1, rd, op_id.0, op_id.1, op_id.2);
}
