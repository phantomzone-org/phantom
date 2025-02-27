//! JAL
//!
//! # Format
//!
//! jal rd,offset
//!
//! # Description
//!
//! Jump to address and place return address in rd.
//!
//! Implementation
//!
//! jal    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = pc + 4; pc += sext(imm[19:0])

use super::{decomp, reconstruct, sext, PcUpdates};
use crate::parameters::{U20DECOMP, U32DECOMP};

pub struct Jal();

impl PcUpdates for Jal {
    fn apply(
        &self,
        imm: &[u32],
        _x_rs1: &[u32],
        _x_rs2: &[u32],
        pc: &[u32],
    ) -> (Vec<u32>, Vec<u32>) {
        (
            decomp(reconstruct(pc, &U32DECOMP) + 4, &U32DECOMP),
            decomp(
                reconstruct(pc, &U32DECOMP).wrapping_add(sext(reconstruct(imm, &U20DECOMP), 20)),
                &U32DECOMP,
            ),
        )
    }
}

#[cfg(test)]
use crate::instructions::{encode_1101111, Instructions};
#[test]
fn instruction_parsing() {
    // imm[20|10:1|11|19:12] | rd[11:7] | 11011 | 11

    // imm = 0b10101100110111110000 | 0
    // 1) split [20] [19:12] [11] [10:1]: 1 01011001  1   0111110000 | 0
    // 2) rearrange [19] [10:1] [11] [19:12]: 1 0111110000  1  01011001  | 0

    let imm_19: u8 = 0b1010; // 20 19 18 17
    let imm_15: u8 = 0b1100; // 16 15 14 13
    let imm_11: u8 = 0b1101; // 12 11 10  9
    let imm_7: u8 = 0b1111; //  8  7  6  5
    let imm_3: u8 = 0b0000; //  4  3  2  1
    let rs2: u8 = 0;
    let rs1: u8 = 0;
    let rd: u8 = 00001;
    let rd_w: u8 = 27;
    let mem_w: u8 = 0;
    let pc_w: u8 = 1;

    let rv32: u32 = encode_1101111(imm_19, imm_15, imm_11, imm_7, imm_3, rd);
    assert_eq!(rv32, 0xbe1590ef);

    let mut m: Instructions = Instructions::new();
    m.add(rv32);
    m.assert_size(1);
    m.assert_instruction(
        0, imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    );
}
