// use base2k::{Module, MODULETYPE};
// use fhevm::decompose::{Base1D, Decomposer, Precomp};

// #[test]
// fn decompose_u32() {
//     let n: usize = 1 << 12;
//     let log_base2k: usize = 15;
//     let cols: usize = 4;
//     let module_pbs: Module = Module::new(n, MODULETYPE::FFT64);

//     let base_1d: Base1D = Base1D(vec![6, 6, 6, 6, 6, 2]);

//     let mut decomposer: Decomposer = Decomposer::new(&module_pbs, cols);
//     let precomp: Precomp = Precomp::new(n, &base_1d, log_base2k, cols);

//     let value: u32 = 0xf0f0f0ff;

//     let result: Vec<u8> = decomposer.decompose(&module_pbs, &precomp, value);

//     assert_eq!(value, base_1d.recomp(&result));
// }
