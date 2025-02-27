//! JALR
//!
//! # Format
//!
//! jalr rd,rs1,offset
//!
//! # Description
//!
//! Jump to address and place return address in rd.
//!
//! Implementation
//!
//! jalr   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | t = pc + 4; pc = (x[rs1] + sext(imm[11:0])) & ~1; x[rd] = t

use super::{decomp, reconstruct, sext, PcUpdates};
use crate::parameters::{U12DECOMP, U32DECOMP};

pub struct Jalr();

impl PcUpdates for Jalr {
    fn apply(
        &self,
        imm: &[u32],
        x_rs1: &[u32],
        _x_rs2: &[u32],
        pc: &[u32],
    ) -> (Vec<u32>, Vec<u32>) {
        (
            decomp(reconstruct(pc, &U32DECOMP) + 4, &U32DECOMP),
            decomp(
                reconstruct(x_rs1, &U32DECOMP).wrapping_add(sext(reconstruct(imm, &U12DECOMP), 12))
                    & !1,
                &U32DECOMP,
            ),
        )
    }
}

#[cfg(test)]
use crate::instructions::{encode_1100111, Instructions};
#[test]
fn instruction_parsing() {
    // imm[11:0] | rs1[19:15] | 000 | rd[11:7] | 11001 | 11
    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = 0b1011;
    let imm_7: u8 = 0b0111;
    let imm_3: u8 = 0b1001;
    let rs2: u8 = 0;
    let rs1: u8 = 0b1001;
    let rd: u8 = 0b0101;
    let rd_w: u8 = 28;
    let mem_w: u8 = 0;
    let pc_w: u8 = 2;

    let rv32: u32 = encode_1100111(imm_11, imm_7, imm_3, rs1, 0b000, rd);

    let mut m: Instructions = Instructions::new();
    m.add(rv32);
    m.assert_size(1);
    m.assert_instruction(
        0, imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    );
}
