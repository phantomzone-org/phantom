use base2k::{alloc_aligned_u8, Infos, Module, VecZnxApi, FFT64};
use fhevm::address::Address;
use fhevm::memory::{read_prepare_write_tmp_bytes, read_tmp_bytes, write_tmp_bytes, Memory};

#[test]
fn memory() {
    let log_n: usize = 5;
    let n: usize = 1 << log_n;
    let log_q: usize = 54;
    let log_base2k: usize = 15;
    let log_base_n: usize = 6;

    let rows: usize = (log_q + log_base2k - 1) / log_base2k;
    let cols: usize = rows;

    let module = Module::new::<FFT64>(n);

    let size: usize = 2 * n + 1;
    let mut data: Vec<i64> = vec![i64::default(); size];
    data.iter_mut().enumerate().for_each(|(i, x)| *x = i as i64);

    let mut memory: Memory = Memory::new(&module, log_base2k, cols, size);
    memory.set(&data, log_q);
    let mut address: Address = Address::new(&module, log_base_n, size, rows, cols);

    let write_value: i64 = 255;

    let mut tmp_bytes = alloc_aligned_u8(
        read_tmp_bytes(&module, cols, rows, cols)
            | read_prepare_write_tmp_bytes(&module, cols, rows, cols)
            | write_tmp_bytes(&module, cols, rows, cols),
        64,
    );

    (0..size).for_each(|i| {
        //println!("{:?}", i);

        // Sets the address to i
        address.set(&module, i);

        // Read only idx[i] and check it is equal to i
        let value: i64 = memory.read(&module, &address, &mut tmp_bytes);
        assert_eq!(i as i64, value);

        // Reads idx[i] with prepare for write  check it is equal to i
        let value: i64 = memory.read_prepare_write(&module, &address, &mut tmp_bytes);
        assert_eq!(i as i64, value);

        // Writes write_value on idx[i]
        memory.write(&module, &address, write_value, &mut tmp_bytes);

        // Read only idx[i], checks it is equal to write_value.
        let value: i64 = memory.read(&module, &address, &mut tmp_bytes);
        assert_eq!(write_value, value);
    });
}
