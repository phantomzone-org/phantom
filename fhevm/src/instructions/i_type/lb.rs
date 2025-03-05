//! | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = sext(M[x[rs1] + sext(imm[11:0])][7:0])

use crate::address::Address;
use crate::circuit_bootstrapping::CircuitBootstrapper;
use crate::instructions::{decompose, reconstruct, sext, Load};
use crate::memory::Memory;
use base2k::Module;

pub struct Lb();

impl Load for Lb {
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
        let read: u32 = memory.read_prepare_write(module_lwe, address, tmp_bytes) as u32;
        decompose(sext(read & 0xFF, 7))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::Address;
    use crate::circuit_bootstrapping::circuit_bootstrap_tmp_bytes;
    use crate::instructions::{decompose, reconstruct, sext};
    use crate::memory::{read_prepare_write_tmp_bytes, read_tmp_bytes, write_tmp_bytes, Memory};
    use base2k::{alloc_aligned_u8, Module, VecZnxApi, FFT64};

    #[test]
    pub fn apply() {
        let log_n: usize = 6;
        let n: usize = 1 << log_n;
        let n_acc = n << 2;
        let log_q: usize = 54;
        let log_base2k: usize = 15;
        let log_base_n: usize = 6;

        let cols: usize = (log_q + log_base2k - 1) / log_base2k;
        let rows: usize = cols;
        let cols_gct = cols + 1;

        let module_lwe: Module = Module::new::<FFT64>(n);
        let module_pbs: Module = Module::new::<FFT64>(n_acc);

        let size: usize = 2 * n + 1;
        let mut data: Vec<i64> = vec![i64::default(); size];
        data.iter_mut().enumerate().for_each(|(i, x)| *x = i as i64);

        let mut memory: Memory = Memory::new(&module_lwe, log_base2k, cols, size);
        memory.set(&data, log_base2k);

        let mut address: Address = Address::new(&module_lwe, log_base_n, size, rows, cols);

        let mut tmp_bytes = alloc_aligned_u8(
            read_tmp_bytes(&module_lwe, cols, rows, cols)
                | read_prepare_write_tmp_bytes(&module_lwe, cols, rows, cols)
                | write_tmp_bytes(&module_lwe, cols, rows, cols)
                | circuit_bootstrap_tmp_bytes(&module_pbs, cols),
        );

        let circuit_btp: CircuitBootstrapper =
            CircuitBootstrapper::new(&module_pbs, module_lwe.log_n(), log_base2k, cols_gct);

        let imm: u32 = sext(0xF, 11);
        let x_rs1: u32 = n as u32;

        let loaded: [u8; 8] = Lb::apply(
            &module_pbs,
            &module_lwe,
            &decompose(imm),
            &decompose(x_rs1),
            &mut memory,
            &circuit_btp,
            &mut address,
            &mut tmp_bytes,
        );

        assert_eq!(x_rs1.wrapping_add(imm) & 0xFF, reconstruct(&loaded));
    }
}
