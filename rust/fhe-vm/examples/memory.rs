use fhevm::address::Address;
use fhevm::memory::Memory;
use math::ring::Ring;

fn main() {
    let n: usize = 1 << 4;
    let q_base: u64 = 65537u64;
    let q_power: usize = 1usize;
    let ring: Ring<u64> = Ring::new(n, q_base, q_power);
    let size: usize = n * n * n * n;
    let mut data: Vec<u64> = vec![0u64; size];
    data.iter_mut().enumerate().for_each(|(i, x)| *x = i as u64);

    let mut memory: Memory = Memory::new(&ring, &data);
    let mut idx: Address = Address::new(&ring, size);

    let write_value: u64 = 255;

    // Read
    (0..16).for_each(|i| {
        idx.set(&ring, i);

        println!("{}", i);

        // Reads idx[i] check it is equal to i, and writes write_value on idx[i]
        let value = memory.read_and_write(&ring, &idx, write_value, true);
        assert_eq!(i as u64, value);

        // Reads idx[i], checks it is equal to write_value and writes back i on idx[i].
        let value = memory.read_and_write(&ring, &idx, i as u64, true);
        assert_eq!(write_value, value);
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
