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
