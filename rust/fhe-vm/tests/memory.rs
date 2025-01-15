use fhevm::memory::{Index, Memory};
use math::ring::Ring;

#[test]
fn memory() {
    let n: usize = 1 << 4;
    let q_base: u64 = 65537u64;
    let q_power: usize = 1usize;
    let ring: Ring<u64> = Ring::new(n, q_base, q_power);
    let size: usize = n * n;
    let mut data: Vec<u64> = vec![0u64; size];
    data.iter_mut().enumerate().for_each(|(i, x)| *x = i as u64);

    let mut memory: Memory = Memory::new(&ring, &data);
    let mut idx = Index::new(&ring, size);

    // Read
    (0..size).for_each(|i| {
        idx.set(&ring, i);
        let value = memory.read_and_write(&ring, &idx, 0, false);
        assert_eq!(i as u64, value);
    });

    // Write
    /*
    let write = 1;
    let value_write  = 37;
    idx.set(&ring, 255);
    memory.write(&ring, value_write, write, &idx);

    let value_read = memory.read(&ring, &idx);
    println!("write: {} read: {}", value_write, value_read);
     */
}
