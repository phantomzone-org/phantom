use fhevm::memory::{Index, Memory};
use math::ring::Ring;

#[test]
fn memory() {
    let n: usize = 1 << 4;
    let q_base: u64 = 65537u64;
    let q_power: usize = 1usize;
    let ring: Ring<u64> = Ring::new(n, q_base, q_power);
    let size: usize = n*n;
    let mut data: Vec<u64> = vec![0u64; size];
    data.iter_mut().enumerate().for_each(|(i, x)| *x = i as u64);

    let memory: Memory = Memory::new(&ring, &data);
    let mut idx = Index::new(&ring, size);

    (0..size).for_each(|i| {
        idx.set(&ring, i);
        let value = memory.read(&ring, &idx);
        assert_eq!(i as u64, value);
    })
}
