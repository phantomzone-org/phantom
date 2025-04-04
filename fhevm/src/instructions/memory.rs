use crate::address::Address;
use crate::circuit_bootstrapping::CircuitBootstrapper;
use crate::decompose::{Decomposer, Precomp};
use crate::instructions::{decompose, reconstruct};
use crate::memory::Memory;
use base2k::Module;

// Address = X^{x_rs1 + imm}
pub fn prepare_address(
    module_pbs: &Module,
    module_lwe: &Module,
    imm: &[u8; 8],
    x_rs1: &[u8; 8],
    circuit_btp: &CircuitBootstrapper,
    decomposer: &mut Decomposer,
    precomp: &Precomp,
    address: &mut Address,
    tmp_bytes: &mut [u8],
) {
    let imm_u32: u32 = reconstruct(imm);
    let x_rs1_u32: u32 = reconstruct(x_rs1);
    let idx: u32 = x_rs1_u32.wrapping_add(imm_u32);
    circuit_btp.bootstrap_to_address(
        module_pbs, module_lwe, decomposer, precomp, idx, address, tmp_bytes,
    );
}

// Address = X^{(x_rs1 + imm) - (x_rs1+imm)%4}
// Offset = (x_rs1+imm)%4
pub fn prepare_address_floor_byte_offset(
    module_pbs: &Module,
    module_lwe: &Module,
    imm: &[u8; 8],
    x_rs1: &[u8; 8],
    circuit_btp: &CircuitBootstrapper,
    decomposer: &mut Decomposer,
    precomp_byte_offset: &Precomp,
    precomp_address: &Precomp,
    address: &mut Address,
    tmp_bytes: &mut [u8],
) -> u8 {
    let imm_u32: u32 = reconstruct(imm);
    let x_rs1_u32: u32 = reconstruct(x_rs1);
    let mut idx: u32 = x_rs1_u32.wrapping_add(imm_u32);
    let offset: u32 = decomposer.decompose(module_pbs, precomp_byte_offset, idx)[0] as u32;
    assert_eq!(idx & 3, offset);
    assert!(offset != 1, "invalid offset: 1");
    idx -= offset;
    circuit_btp.bootstrap_to_address(
        module_pbs,
        module_lwe,
        decomposer,
        precomp_address,
        idx,
        address,
        tmp_bytes,
    );
    offset as u8
}

pub fn load(
    module_lwe: &Module,
    memory: &mut Memory,
    address: &mut Address,
    tmp_bytes: &mut [u8],
) -> [u8; 8] {
    decompose(memory.read_prepare_write(module_lwe, address, tmp_bytes))
}

pub fn store(
    module_lwe: &Module,
    value: &[u8; 8],
    memory: &mut Memory,
    address: &mut Address,
    tmp_bytes: &mut [u8],
) {
    memory.write(module_lwe, &address, reconstruct(value), tmp_bytes)
}

// TODO: bootstrapp offset to address
pub fn extract_from_byte_offset(value: &[u8; 8], offset: u8) -> [u8; 8] {
    assert!(offset != 1, "invalid offset: 1");
    let values: [[u8; 8]; 4] = [
        *value,
        [0, 0, 0, 0, 0, 0, 0, 0], // Never used since offset is 0, 2, 3 (4, 2, 1 bytes)
        [value[4], value[5], value[6], value[7], 0, 0, 0, 0],
        [value[6], value[7], 0, 0, 0, 0, 0, 0],
    ];
    values[offset as usize]
}

// TODO: bootstrapp offset to address
pub fn select_store(to_store: &[u8; 8], loaded: &[u8; 8], offset: u8) -> [u8; 8] {
    assert!(offset != 1, "invalid offset: 1");
    let values: [[u8; 8]; 4] = [
        *to_store,
        [0, 0, 0, 0, 0, 0, 0, 0], // Never used since offset is 0, 2, 3 (4, 2, 1 bytes)
        [
            loaded[0],
            loaded[1],
            loaded[2],
            loaded[3],
            to_store[0],
            to_store[1],
            to_store[2],
            to_store[3],
        ],
        [
            loaded[0],
            loaded[1],
            loaded[2],
            loaded[3],
            loaded[4],
            loaded[5],
            to_store[0],
            to_store[1],
        ],
    ];
    values[offset as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::Address;
    use crate::circuit_bootstrapping::circuit_bootstrap_tmp_bytes;
    use crate::instructions::{decompose, reconstruct, sext};
    use crate::memory::{read_prepare_write_tmp_bytes, read_tmp_bytes, write_tmp_bytes, Memory};
    use crate::parameters::DECOMPOSE_BYTEOFFSET;
    use base2k::{alloc_aligned_u8, Module, MODULETYPE};

    #[test]
    pub fn apply() {
        let log_n: usize = 6;
        let n: usize = 1 << log_n;
        let n_acc = n << 2;
        let log_q: usize = 54;
        let log_base2k: usize = 17;

        let cols: usize = (log_q + log_base2k - 1) / log_base2k;
        let rows: usize = cols;
        let cols_gct = cols + 1;

        let module_lwe: Module = Module::new(n, MODULETYPE::FFT64);
        let module_pbs: Module = Module::new(n_acc, MODULETYPE::FFT64);

        let size: usize = 2 * n + 1;
        let mut data: Vec<i64> = vec![i64::default(); size];
        data.iter_mut()
            .enumerate()
            .for_each(|(i, x)| *x = 0xFFFF_FFFF - i as i64);

        let mut memory: Memory = Memory::new(&module_lwe, log_base2k, cols, size);
        memory.set(&data, 2 * log_base2k);

        let address_decomp: Vec<Vec<u8>> = vec![vec![3, 3], vec![3, 3]];

        let mut address: Address = Address::new(&module_lwe, address_decomp, rows, cols);

        let mut tmp_bytes = alloc_aligned_u8(
            read_tmp_bytes(&module_lwe, cols, rows, cols)
                | read_prepare_write_tmp_bytes(&module_lwe, cols, rows, cols)
                | write_tmp_bytes(&module_lwe, cols, rows, cols)
                | circuit_bootstrap_tmp_bytes(&module_pbs, cols),
        );

        let circuit_btp: CircuitBootstrapper =
            CircuitBootstrapper::new(&module_pbs, module_lwe.log_n(), log_base2k, cols_gct);

        let mut decomposer: Decomposer = Decomposer::new(&module_pbs, cols);

        let precomp_byte_offset: Precomp = Precomp::new(
            module_pbs.n(),
            &DECOMPOSE_BYTEOFFSET.to_vec(),
            log_base2k,
            cols,
        );
        let precomp_address: Precomp = Precomp::new(
            module_pbs.n(),
            &address.decomp_flattened(),
            log_base2k,
            cols,
        );

        for i in [0, 2, 3] {
            println!("i: {}", i);
            let imm: u32 = sext(0xC + i, 11);
            let x_rs1: u32 = n as u32;

            let offset: u8 = prepare_address_floor_byte_offset(
                &module_pbs,
                &module_lwe,
                &decompose(imm),
                &decompose(x_rs1),
                &circuit_btp,
                &mut decomposer,
                &precomp_byte_offset,
                &precomp_address,
                &mut address,
                &mut tmp_bytes,
            );

            println!("offset: {}", offset);

            assert_eq!((x_rs1.wrapping_add(imm) & 3) as u8, offset);

            let value_full: [u8; 8] = load(&module_lwe, &mut memory, &mut address, &mut tmp_bytes);

            println!();
            assert_eq!(
                data[(x_rs1.wrapping_add(imm) - offset as u32) as usize] as u32,
                reconstruct(&value_full)
            );

            let loaded_offset: [u8; 8] = extract_from_byte_offset(&value_full, offset);

            assert_eq!(
                (data[(x_rs1.wrapping_add(imm) - offset as u32) as usize] >> (8 * offset)) as u32,
                reconstruct(&loaded_offset)
            );

            let shift: u32 = (offset << 3) as u32;

            let value: u32 = 0xAABB_CCDD >> shift;

            let to_store: [u8; 8] = select_store(&decompose(value), &value_full, offset);

            //println!("to_store: {:08x}", reconstruct(&to_store));

            store(
                &module_lwe,
                &to_store,
                &mut memory,
                &mut address,
                &mut tmp_bytes,
            );

            let have: u32 = memory.read(&module_lwe, &address, &mut tmp_bytes);
            let shift_inv: u32 = 32 - shift;
            let value_lh: u32 = value << shift;
            let value_rh: u32 = ((((reconstruct(&value_full) as u64) << shift_inv) as u32 as u64)
                >> shift_inv) as u32;
            let want: u32 = value_lh | value_rh;

            // Need to update local reference memory
            data[(x_rs1.wrapping_add(imm) - offset as u32) as usize] = want as i64;

            //println!("value_lh: {:08x}", value_lh);
            //println!("value_rh: {:08x}", value_rh);
            //println!("have: {:08x}", have);

            assert_eq!(have, want);
        }
    }
}
