//! sb     | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | M[x[rs1] + sext(imm[11:0])] = x[rs2][7:0]

use super::{reconstruct, sext, Store};
use crate::address::Address;
use crate::circuit_bootstrapping::CircuitBootstrapper;
use crate::memory::Memory;
use crate::parameters::{U12DECOMP, U32DECOMP};
use base2k::Module;

pub struct Sb();

impl Store for Sb {
    fn apply(
        &self,
        module_pbs: &Module,
        module_lwe: &Module,
        imm: &[u32],
        x_rs1: &[u32],
        _memory: &mut Memory,
        circuit_btp: &CircuitBootstrapper,
        address: &mut Address,
        tmp_bytes: &mut [u8],
    ) {
        let x_rs1_u32: u32 = reconstruct(x_rs1, &U32DECOMP);
        let imm_u32: u32 = sext(reconstruct(imm, &U12DECOMP), 12);
        let idx: u32 = x_rs1_u32.wrapping_add(imm_u32);
        circuit_btp.bootstrap_to_address(module_pbs, module_lwe, idx, address, tmp_bytes);
        //memory.write
    }
}

#[cfg(test)]
use crate::instructions::{encode_0100011, Instructions};
#[test]
fn instruction_parsing() {
    // imm[11:5] | rs2[24:20] | rs1[19:15] | 000 | imm[4:0] | 01000 | 11
    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = 0b1101;
    let imm_7: u8 = 0b1001;
    let imm_3: u8 = 0b1011;
    let rs2: u8 = 0b11011;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0;
    let rd_w: u8 = 0;
    let mem_w: u8 = 1;
    let pc_w: u8 = 0;

    let rv32: u32 = encode_0100011(imm_11, imm_7, imm_3, rs2, rs1, 0b000);

    let mut m: Instructions = Instructions::new();
    m.add(rv32);
    m.assert_size(1);
    m.assert_instruction(
        0, imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    );
}
