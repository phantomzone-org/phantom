use fhevm::address::Address;
use fhevm::memory::Memory;
use fhevm::parameters::{GADGETDECOMP, LOGNLWE};
use math::ring::Ring;

#[test]
fn memory() {
    let n: usize = 1 << 7;
    let q_base: u64 = 0x1fffffffffe00001;
    let q_power: usize = 1usize;
    let ring: Ring<u64> = Ring::new(n, q_base, q_power);
    let size: usize = 2 * n - 37;
    let mut data: Vec<u64> = vec![0u64; size];
    data.iter_mut().enumerate().for_each(|(i, x)| *x = i as u64);

    let mut memory: Memory = Memory::new(&ring);
    memory.set(&ring, &data);
    let mut idx = Address::new(&ring, GADGETDECOMP, 6, size);

    let write_value: u64 = 255;

    // Read & Write
    (0..size).for_each(|i| {
        idx.set(&ring, i);

        //println!("{:?}", i);

        // Reads idx[i] check it is equal to i.
        let value: u64 = memory.read(&ring, &idx);
        assert_eq!(i as u64, value);
    });

    // Read & Write
    (0..size).for_each(|i| {
        idx.set(&ring, i);

        //println!("{:?}", i);

        // Reads idx[i] check it is equal to i, and writes write_value on idx[i]
        let value: u64 = memory.read_and_write(&ring, &idx, write_value, true);
        assert_eq!(i as u64, value);

        // Reads idx[i], checks it is equal to write_value and writes back i on idx[i].
        let value: u64 = memory.read_and_write(&ring, &idx, i as u64, true);
        assert_eq!(write_value, value);
    });
}
