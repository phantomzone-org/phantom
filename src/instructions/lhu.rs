//! lhu    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = M[x[rs1] + sext(imm[11:0])][15:0]

use super::{decomp, reconstruct, sext, Load};
use crate::address::Address;
use crate::circuit_bootstrapping::CircuitBootstrapper;
use crate::memory::Memory;
use crate::parameters::{U12DECOMP, U16DECOMP, U32DECOMP};
use base2k::Module;

pub struct Lhu();

impl Load for Lhu {
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
        decomp(read & 0xFFFF, &U16DECOMP)
    }
}
