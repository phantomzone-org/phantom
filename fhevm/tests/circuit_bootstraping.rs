use base2k::{
    alloc_aligned_u8, Encoding, Module, VecZnx, VecZnxBig, VecZnxBigOps, VecZnxDft, VecZnxDftOps,
    VecZnxOps, VmpPMat, VmpPMatOps, MODULETYPE,
};
use fhevm::circuit_bootstrapping::{circuit_bootstrap_tmp_bytes, CircuitBootstrapper};
use itertools::izip;

#[test]
fn circuit_bootstrapping() {
    let log_n = 5;
    let n: usize = 1 << log_n;
    let n_acc = n << 2;
    let log_base2k: usize = 17;
    let cols: usize = 4;
    let log_k: usize = cols * log_base2k - 5;
    let module_lwe: Module = Module::new(n, MODULETYPE::FFT64);

    let module_pbs: Module = Module::new(n_acc, MODULETYPE::FFT64);

    let gct_rows: usize = cols;
    let gct_cols: usize = cols + 1;

    let acc: CircuitBootstrapper = CircuitBootstrapper::new(log_base2k, gct_cols);

    let mut vec_gadget: Vec<VecZnx> = Vec::new();
    (0..gct_cols).for_each(|_| {
        vec_gadget.push(module_lwe.new_vec_znx(gct_cols));
    });

    let mut vmp_pmat: VmpPMat = module_lwe.new_vmp_pmat(gct_rows, gct_cols);

    let max_value: usize = std::cmp::min(u8::MAX as usize, n - 1);

    let mut tmp_bytes: Vec<u8> = alloc_aligned_u8(
        circuit_bootstrap_tmp_bytes(&module_pbs, cols)
            | module_lwe.vmp_apply_dft_tmp_bytes(cols, cols, gct_rows, gct_cols)
            | module_lwe.vmp_prepare_tmp_bytes(gct_rows, gct_cols),
    );

    // value in [0, n_acc/2^{log_gap} - 1]
    (0..max_value + 1).for_each(|value| {
        // Maps value in [0, 2^{log_n}] to X^{value * 2^{log_max_drift+1-(log_n_pbs-log_n_lwe)} +/- floor(drift/2^{log_max_drift})}
        let log_gap_in = acc.circuit_bootstrap(
            &module_pbs,
            value as u8,
            log_n,
            &mut vec_gadget,
            &mut tmp_bytes,
        );

        let log_gap_out: usize = log_gap_in;

        // Maps X^(i * 2^{log_gap_in}) to X^(i * 2^{log_gal_out})
        acc.post_process(
            &module_lwe,
            log_gap_in as u8,
            log_gap_out as u8,
            max_value,
            &mut vec_gadget,
            &mut tmp_bytes,
        );

        (0..gct_rows).for_each(|row_i| {
            module_lwe.vmp_prepare_row(
                &mut vmp_pmat,
                &vec_gadget[row_i].raw(),
                row_i,
                &mut tmp_bytes,
            );
        });

        let mut vec_have: VecZnx = module_lwe.new_vec_znx(cols);
        vec_have.encode_coeff_i64(log_base2k, log_k, 0, 1, 32);
        vec_have.normalize(log_base2k, &mut tmp_bytes);

        //println!("INPUT");
        //(0..vec_have.cols()).for_each(|i|{
        //    println!("{}: {:?}", i, vec_have.at(i))
        //});
        //println!();

        let mut c_dft: VecZnxDft = module_lwe.new_vec_znx_dft(gct_cols);
        module_lwe.vmp_apply_dft(&mut c_dft, &vec_have, &vmp_pmat, &mut tmp_bytes);

        let mut c_big: VecZnxBig = c_dft.as_vec_znx_big();
        module_lwe.vec_znx_idft_tmp_a(&mut c_big, &mut c_dft, gct_cols);

        let mut res: VecZnx = module_lwe.new_vec_znx(gct_cols);
        module_lwe.vec_znx_big_normalize(log_base2k, &mut res, &c_big, &mut tmp_bytes);

        //println!("OUTPUT");
        //(0..res.cols()).for_each(|i|{
        //    println!("{}: {:?}", i, res.at(i))
        //});
        //println!();

        let mut vec_want: VecZnx = module_lwe.new_vec_znx(cols);
        vec_want.encode_coeff_i64(log_base2k, log_k, value << log_gap_out, 1, 2);

        let mut have: Vec<i64> = vec![i64::default(); module_lwe.n()];
        let mut want: Vec<i64> = vec![i64::default(); module_lwe.n()];

        res.decode_vec_i64(log_base2k, log_k, &mut have);
        vec_want.decode_vec_i64(log_base2k, log_k, &mut want);

        //println!("{:?}", want);
        //println!("{:?}", have);

        izip!(want, have).for_each(|(a, b)| assert_eq!(a, b));
    });
}
