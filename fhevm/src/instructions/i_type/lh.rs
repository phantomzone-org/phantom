//! lh     | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = sext(M[x[rs1] + sext(imm[11:0])][15:0])

use crate::address::Address;
use crate::circuit_bootstrapping::CircuitBootstrapper;
use crate::instructions::{decompose, reconstruct, sext, Load};
use crate::memory::Memory;
use base2k::Module;

pub struct Lh();

impl Load for Lh {
    fn apply(
        module_pbs: &Module,
        module_lwe: &Module,
        imm: &[u8; 8],
        x_rs1: &[u8; 8],
        memory: &mut Memory,
        circuit_btp: &CircuitBootstrapper,
        address: &mut Address,
        tmp_bytes: &mut [u8],
    ) -> [u8; 8] {
        let imm_u32: u32 = reconstruct(imm);
        let x_rs1_u32: u32 = reconstruct(x_rs1);
        let idx: u32 = x_rs1_u32.wrapping_add(imm_u32);
        circuit_btp.bootstrap_to_address(module_pbs, module_lwe, idx, address, tmp_bytes);
        let read: u32 = memory.read(module_lwe, address, tmp_bytes) as u32;
        decompose(sext(read & 0xFFFF, 15))
    }
}
