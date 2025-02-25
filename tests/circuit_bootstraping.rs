use base2k::{
    alloc_aligned_u8, Encoding, Module, VecZnx, VecZnxApi, VecZnxBig, VecZnxBigOps, VecZnxDft,
    VecZnxDftOps, VecZnxOps, VecZnxVec, VmpPMat, VmpPMatOps, FFT64,
};
use fhevm::circuit_bootstrapping::{circuit_bootstrap_tmp_bytes, CircuitBootstrapper};
use itertools::izip;

#[test]
fn circuit_bootstrapping() {
    let n: usize = 1 << 5;
    let n_acc = n << 2;
    let log_base2k: usize = 17;
    let limbs: usize = 4;
    let log_k: usize = limbs * log_base2k - 5;
    let module_lwe: Module = Module::new::<FFT64>(n);

    let module_pbs: Module = Module::new::<FFT64>(n_acc);
    let log_gap: usize = 3;

    let rows: usize = limbs;
    let cols: usize = limbs + 1;

    let acc: CircuitBootstrapper =
        CircuitBootstrapper::new(&module_pbs, module_lwe.log_n(), log_base2k, cols);

    let mut vec_gadget: Vec<VecZnx> = Vec::new();
    (0..cols).for_each(|_| {
        vec_gadget.push(module_lwe.new_vec_znx(cols));
    });

    let mut vmp_pmat: VmpPMat = module_lwe.new_vmp_pmat(rows, cols);

    let max_value: usize = n;

    let mut tmp_bytes: Vec<u8> = alloc_aligned_u8(
        circuit_bootstrap_tmp_bytes(&module_pbs, limbs)
            | module_lwe.vmp_apply_dft_tmp_bytes(limbs, limbs, rows, cols)
            | module_lwe.vmp_prepare_tmp_bytes(rows, cols),
    );

    // value in [0, n_acc/2^{log_gap} - 1]
    (0..n_acc / (1 << log_gap)).for_each(|value| {
        // value in [0, n_acc - 2^log_gap]
        let value_scaled: i64 = (value << log_gap) as i64;

        // Maps value in [0, n_acc - 2^log_gap] to X^{value * (2^log_gap*n/n_acc) +/- drift/2^{log_gap-1}}
        acc.circuit_bootstrap(&module_pbs, value_scaled, &mut vec_gadget, &mut tmp_bytes);

        let log_gap_in: usize = log_gap - (module_pbs.log_n() - module_lwe.log_n());
        let log_gap_out: usize = log_gap_in;

        //println!("log_gap_in: {}", log_gap_in);
        //println!("log_gap_out: {}", log_gap_out);

        // Maps X^(i * 2^{log_gap_in}) to X^(i * 2^{log_gal_out})
        acc.post_process(
            &module_lwe,
            log_gap_in,
            log_gap_out,
            max_value,
            &mut vec_gadget,
            &mut tmp_bytes,
        );

        (0..rows).for_each(|row_i| {
            module_lwe.vmp_prepare_row(
                &mut vmp_pmat,
                &vec_gadget[row_i].raw(),
                row_i,
                &mut tmp_bytes,
            );
        });

        let mut vec_have: VecZnx = module_lwe.new_vec_znx(limbs);
        vec_have.encode_coeff_i64(log_base2k, log_k, 0, 1, 32);
        vec_have.normalize(log_base2k, &mut tmp_bytes);

        //println!("INPUT");
        //(0..vec_have.limbs()).for_each(|i|{
        //    println!("{}: {:?}", i, vec_have.at(i))
        //});
        //println!();

        let mut c_dft: VecZnxDft = module_lwe.new_vec_znx_dft(cols);
        module_lwe.vmp_apply_dft(&mut c_dft, &vec_have, &vmp_pmat, &mut tmp_bytes);

        let mut c_big: VecZnxBig = c_dft.as_vec_znx_big();
        module_lwe.vec_znx_idft_tmp_a(&mut c_big, &mut c_dft, cols);

        let mut res: VecZnx = module_lwe.new_vec_znx(cols);
        module_lwe.vec_znx_big_normalize(log_base2k, &mut res, &c_big, &mut tmp_bytes);

        //println!("OUTPUT");
        //(0..res.limbs()).for_each(|i|{
        //    println!("{}: {:?}", i, res.at(i))
        //});
        //println!();

        let mut vec_want: VecZnx = module_lwe.new_vec_znx(limbs);
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
