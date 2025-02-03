use base2k::{Module, FFT64};
use fhevm::address::Address;
use fhevm::memory::Memory;

fn main() {
    let log_n: usize = 4;
    let n: usize = 1 << log_n;
    let limbs: usize = 4;
    let log_base2k: usize = 15;
    let log_k = limbs * log_base2k - 5;
    let log_base_n: usize = 7;

    let rows: usize = limbs;
    let cols: usize = limbs + 1;

    let module = Module::new::<FFT64>(n);

    let size: usize = n * n * n * n;
    let mut data: Vec<i64> = vec![i64::default(); size];
    data.iter_mut().enumerate().for_each(|(i, x)| *x = i as i64);

    let mut memory: Memory = Memory::new(log_n, log_base2k, log_k);
    memory.set(&data);
    let mut idx: Address = Address::new(&module, log_base_n, size, rows, cols);

    let write_value: i64 = 255;

    // Read
    (0..16).for_each(|i| {
        idx.set(&module, i);

        println!("{}", i);

        // Reads idx[i] check it is equal to i, and writes write_value on idx[i]
        let value = memory.read_and_write(&module, &idx, write_value, true);
        assert_eq!(i as i64, value);

        // Reads idx[i], checks it is equal to write_value and writes back i on idx[i].
        let value = memory.read_and_write(&module, &idx, i as i64, true);
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
