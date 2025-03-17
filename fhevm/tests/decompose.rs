use base2k::{Module, MODULETYPE};
use fhevm::decompose::Decomposer;

#[test]
fn decompose_u32() {
    let n: usize = 1 << 12;
    let log_base2k: usize = 15;
    let limbs: usize = 4;
    let module = Module::new(n, MODULETYPE::FFT64);

    let log_base: Vec<u8> = [6, 6, 6, 6, 6, 2].to_vec();

    let mut decomposer: Decomposer = Decomposer::new(&module, &log_base, log_base2k, limbs);

    let value: u32 = 0xf0f0f0ff;

    let result: Vec<i64> = decomposer.decompose(&module, value);

    let mut have: u32 = 0;

    let mut sum_bases: u8 = 0;
    log_base.iter().enumerate().for_each(|(i, base)| {
        have |= (result[i] << sum_bases) as u32;
        sum_bases += base;
    });
    assert_eq!(value, have);
}
