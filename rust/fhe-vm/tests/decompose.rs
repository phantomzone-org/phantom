use fhevm::decompose::Decomposer;
use fhevm::parameters::ADDRESSBASE;
use rns::ring::Ring;

/*
#[test]
fn decompose_u32() {
    let n: usize = 1 << 12;
    let q_base: u64 = 65537;
    let q_power: usize = 1usize;
    let ring: Ring<u64> = Ring::new(n, q_base, q_power);

    let log_base: Vec<usize> = [6, 6, 6, 6, 6, 2].to_vec();

    let mut decomposer: Decomposer = Decomposer::new(&ring, &log_base);

    let value: u32 = 0xf0f0f0ff;

    let result: Vec<u64> = decomposer.decompose(&ring, value);

    println!("{:?}", result);

    let mut have: u32 = 0;

    let mut sum_bases: usize = 0;
    log_base.iter().enumerate().for_each(|(i, base)| {
        have |= (result[i] << sum_bases) as u32;
        sum_bases += base;
    });
    assert_eq!(value, have);
}
*/
