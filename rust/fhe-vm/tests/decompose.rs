use fhevm::decompose::Decomposer;
use math::ring::Ring;

#[test]
fn decompose_u32() {
    let n: usize = 1 << 12;
    let q_base: u64 = 65537;
    let q_power: usize = 1usize;
    let ring: Ring<u64> = Ring::new(n, q_base, q_power);

    let log_base: usize = 5;
    let d: usize = (32 + log_base - 1) / log_base;

    let mut decomposer: Decomposer = Decomposer::new(&ring, log_base);

    let value: u32 = 0xf0f0f0ff;

    let result: Vec<u64> = decomposer.decompose(&ring, value);

    println!("{:?}", result);

    let mut have: u32 = 0;

    (0..d).for_each(|i| {
        have <<= log_base;
        have |= result[i] as u32;
    });

    assert_eq!(value, have);
}
