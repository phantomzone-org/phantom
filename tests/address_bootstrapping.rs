use base2k::{
    Module, VecZnx, VecZnxOps, FFT64,
};
use fhevm::address::Address;
use fhevm::memory::Memory;
use fhevm::circuit_bootstrapping::CircuitBootstrapper;

#[test]
fn address_bootstrapping() {
    let n_lwe: usize = 1 << 11;
    let n_pbs = n_lwe << 2;
    let log_base2k: usize = 17;
    let limbs: usize = 3;
    let log_base_n: usize = 6;
    let max_address: usize = 2 * n_lwe - 37;
    let module_lwe: Module = Module::new::<FFT64>(n_lwe);

    let module_pbs: Module = Module::new::<FFT64>(n_pbs);

    let rows: usize = limbs;
    let cols: usize = limbs + 1;

    let acc: CircuitBootstrapper =
        CircuitBootstrapper::new(&module_pbs, module_lwe.log_n(), log_base2k, cols);

    let mut address: Address = Address::new(&module_lwe, log_base_n, max_address, rows, cols);

    address.set(&module_lwe, 73);

    let offset: u32 = 0;

    let mut buf_pbs: VecZnx = module_pbs.new_vec_znx(cols);

    acc.bootstrap_address(
        &module_pbs,
        &module_lwe,
        offset,
        max_address,
        &mut address,
        &mut buf_pbs,
    );

    let mut data: Vec<i64> = vec![i64::default(); max_address];
    data.iter_mut().enumerate().for_each(|(i, x)| *x = i as i64);
    let log_k = limbs*log_base2k-5;
    let mut memory: Memory = Memory::new(module_lwe.log_n(), log_base2k, log_k);
    memory.set(&data);
    
    let out: i64 = memory.read(&module_lwe, &address);

    println!("{}", out);
}
