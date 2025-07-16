// use base2k::{Encoding, Module, VecZnx, VecZnxOps, MODULETYPE};
// use fhevm::packing::StreamRepacker;
// use fhevm::reverse_bits_msb;

// #[test]
// fn packing_streaming_u64() {
//     let n: usize = 1 << 4;
//     let module: Module = Module::new(n, MODULETYPE::FFT64);
//     let log_base2k: usize = 16;
//     let log_q: usize = 54;

//     sub_test("test_packing_streaming_dense", || {
//         test_packing_streaming_dense(&module, log_base2k, log_q)
//     });
// }

// fn sub_test<F: FnOnce()>(name: &str, f: F) {
//     println!("Running {}", name);
//     f();
// }

// fn test_packing_streaming_dense(module: &Module, log_base2k: usize, limbs: usize) {
//     let n: usize = module.n();
//     let log_n: usize = module.log_n();
//     let log_k = limbs * log_base2k - 5;

//     let mut values: Vec<i64> = vec![0; n];
//     values
//         .iter_mut()
//         .enumerate()
//         .for_each(|(i, x)| *x = (i + 1) as i64 * 2);

//     let gap: usize = 3;

//     let mut packer = StreamRepacker::new(module, log_base2k, limbs);

//     let mut results: Vec<VecZnx> = Vec::new();

//     let mut tmp: VecZnx = module.new_vec_znx(limbs);
//     for i in 0..n {
//         let i_rev: usize = reverse_bits_msb(i, log_n as u32);
//         if i_rev % gap == 0 {
//             tmp.encode_vec_i64(log_base2k, log_k, &vec![values[i_rev]; n], 32);
//             packer.add(module, Some(&tmp), &mut results)
//         } else {
//             packer.add(module, None::<&VecZnx>, &mut results)
//         }
//     }

//     packer.flush(module, &mut results);

//     let mut have: Vec<i64> = vec![0; n];
//     results[0].decode_vec_i64(log_base2k, log_k, &mut have);

//     println!("{:?}", have);

//     have.iter().enumerate().for_each(|(i, x)| {
//         if i % gap == 0 {
//             assert_eq!(*x, values[i])
//         } else {
//             assert_eq!(*x, 0i64)
//         }
//     });
// }
