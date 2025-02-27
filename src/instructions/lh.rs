//! lh     | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = sext(M[x[rs1] + sext(imm[11:0])][15:0])

use super::{decomp, reconstruct, sext, Load};
use crate::address::Address;
use crate::circuit_bootstrapping::CircuitBootstrapper;
use crate::memory::Memory;
use crate::parameters::{U12DECOMP, U16DECOMP, U32DECOMP};
use base2k::Module;

pub struct Lh();

impl Load for Lh {
    fn apply(
        &self,
        module_pbs: &Module,
        module_lwe: &Module,
        imm: &[u32],
        x_rs1: &[u32],
        memory: &mut Memory,
        circuit_btp: &CircuitBootstrapper,
        address: &mut Address,
        tmp_bytes: &mut [u8],
    ) -> Vec<u32> {
        let imm_u32: u32 = sext(reconstruct(imm, &U12DECOMP), 20);
        let x_rs1_u32: u32 = reconstruct(x_rs1, &U32DECOMP);
        let idx: u32 = x_rs1_u32.wrapping_add(imm_u32);
        circuit_btp.bootstrap_to_address(module_pbs, module_lwe, idx, address, tmp_bytes);
        let read: u32 = memory.read(module_lwe, address, tmp_bytes) as u32;
        decomp(sext(read & 0xFFFF, 16), &U16DECOMP)
    }
}

#[cfg(test)]
use crate::instructions::{encode_0000011, Instructions};
#[test]
fn instruction_parsing() {
    // imm[11:0] | rs1[19:15] | 001 | rd[11:7] | 00000 | 11
    let imm_19: u8 = 0;
    let imm_15: u8 = 0;
    let imm_11: u8 = 0b1011;
    let imm_7: u8 = 0b0111;
    let imm_3: u8 = 0b1001;
    let rs2: u8 = 0;
    let rs1: u8 = 0b10011;
    let rd: u8 = 0b01011;
    let rd_w: u8 = 23;
    let mem_w: u8 = 0;
    let pc_w: u8 = 0;

    let rv32: u32 = encode_0000011(imm_11, imm_7, imm_3, rs1, 0b001, rd);

    let mut m: Instructions = Instructions::new();
    m.add(rv32);
    m.assert_size(1);
    m.assert_instruction(
        0, imm_19, imm_15, imm_11, imm_7, imm_3, rs2, rs1, rd, rd_w, mem_w, pc_w,
    );
}
