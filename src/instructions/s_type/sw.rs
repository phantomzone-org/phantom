use crate::address::Address;
use crate::circuit_bootstrapping::CircuitBootstrapper;
use crate::instructions::{reconstruct, Store};
use crate::memory::Memory;
use base2k::Module;

pub struct Sw();

impl Store for Sw {
    fn apply(
        &self,
        module_pbs: &Module,
        module_lwe: &Module,
        imm: &[u8; 8],
        x_rs1: &[u8; 8],
        x_rs2: &[u8; 8],
        memory: &mut Memory,
        circuit_btp: &CircuitBootstrapper,
        address: &mut Address,
        tmp_bytes: &mut [u8],
    ) {
        let x_rs1_u32: u32 = reconstruct(x_rs1);
        let imm_u32: u32 = reconstruct(imm);
        let idx: u32 = x_rs1_u32.wrapping_add(imm_u32);
        circuit_btp.bootstrap_to_address(module_pbs, module_lwe, idx, address, tmp_bytes);
        memory.write(
            module_lwe,
            &address,
            (reconstruct(x_rs2) & 0xFFFF_FFFF) as u32,
            tmp_bytes,
        )
    }
}
