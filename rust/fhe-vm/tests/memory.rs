use base2k::{Module, FFT64};
use fhevm::address::Address;
use fhevm::memory::Memory;
use fhevm::parameters::{GADGETDECOMP, LOGNLWE};

#[test]
fn memory() {
    let log_n: usize = 7;
    let n: usize = 1 << log_n;
    let log_q: usize = 54;
    let log_base2k: usize = 15;
    let log_base_n: usize = 6;

    let rows: usize = (log_q + log_base2k - 1) / log_base2k;
    let cols: usize = rows;

    let module = Module::new::<FFT64>(n);

    let size: usize = 2*n-37;
    let mut data: Vec<i64> = vec![i64::default(); size];
    data.iter_mut().enumerate().for_each(|(i, x)| *x = i as i64);

    let mut memory: Memory = Memory::new(log_n, log_base2k, log_q);
    memory.set(&data);
    let mut idx: Address = Address::new(&module, log_base_n, size, rows, cols);

    let write_value: i64 = 255;

    // Read & Write
    (0..size).for_each(|i| {
        idx.set(&module, i);

        println!("{:?}", i);

        // Reads idx[i] check it is equal to i.
        let value: i64 = memory.read(&module, &idx);
        assert_eq!(i as i64, value);
    });

    // Read & Write
    (0..size).for_each(|i| {
        idx.set(&module, i);

        println!("{:?}", i);

        // Reads idx[i] check it is equal to i, and writes write_value on idx[i]
        let value: i64 = memory.read_and_write(&module, &idx, write_value, true);
        assert_eq!(i as i64, value);

        // Reads idx[i], checks it is equal to write_value and writes back i on idx[i].
        let value: i64 = memory.read_and_write(&module, &idx, i as i64, true);
        assert_eq!(write_value, value);
    });
}
