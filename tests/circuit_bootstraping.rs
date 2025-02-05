use base2k::{
    Encoding, Module, VecZnx, VecZnxBig, VecZnxDft, VecZnxOps, VmpPMat, VmpPMatOps, FFT64,
};
use fhevm::circuit_bootstrapping::CircuitBootstrapper;
use itertools::izip;

#[test]
fn circuit_bootstrapping() {
    let n: usize = 1 << 5;
    let n_acc = n << 2;
    let log_base2k: usize = 17;
    let limbs: usize = 4;
    let log_k: usize = limbs * log_base2k - 5;
    let module: Module = Module::new::<FFT64>(n);

    let module_acc: Module = Module::new::<FFT64>(n_acc);
    let log_gap: usize = 3;

    let rows: usize = limbs;
    let cols: usize = limbs + 1;

    let acc: CircuitBootstrapper =
        CircuitBootstrapper::new(&module_acc, module.log_n(), log_base2k, cols);
    //acc.init();

    let mut buf_acc: VecZnx = module_acc.new_vec_znx(cols);

    let mut vec_gadget: Vec<VecZnx> = Vec::new();
    (0..cols).for_each(|_| {
        vec_gadget.push(module.new_vec_znx(cols));
    });

    let mut vmp_pmat: VmpPMat = module.new_vmp_pmat(rows, cols);

    // value in [0, n_acc/2^{log_gap} - 1]
    (0..n_acc / (1 << log_gap)).for_each(|value| {
        // value in [0, n_acc - 2^log_gap]
        let value_scaled: i64 = (value << log_gap) as i64;

        // Maps value in [0, n_acc - 2^log_gap] to X^{value * (2^log_gap*n/n_acc) +/- drift/2^{log_gap-1}}
        acc.circuit_bootstrap(&module_acc, value_scaled, &mut buf_acc, &mut vec_gadget);

        let mut buf: [VecZnx; 4] = [
            module.new_vec_znx(cols),
            module.new_vec_znx(cols),
            module.new_vec_znx(cols),
            module.new_vec_znx(cols),
        ];

        let log_gap_in: usize = log_gap - (module_acc.log_n() - module.log_n());
        let log_gap_out: usize = log_gap_in;

        //println!("log_gap_in: {}", log_gap_in);
        //println!("log_gap_out: {}", log_gap_out);

        // Maps X^(i * 2^{log_gap_in}) to X^(i * 2^{log_gal_out})
        acc.post_process(
            &module,
            log_gap_in,
            log_gap_out,
            &mut vmp_pmat,
            &mut vec_gadget,
            &mut buf,
        );

        let tmp_bytes: usize = module.vmp_prepare_contiguous_tmp_bytes(rows, cols)
            | module.vmp_apply_dft_tmp_bytes(limbs, limbs, rows, cols);

        let mut buf: Vec<u8> = vec![0; tmp_bytes];

        let mut vec_have: VecZnx = module.new_vec_znx(limbs);
        vec_have.encode_coeff_i64(log_base2k, log_k, 0, 1, 32);
        vec_have.normalize(log_base2k, &mut buf);

        //println!("INPUT");
        //(0..vec_have.limbs()).for_each(|i|{
        //    println!("{}: {:?}", i, vec_have.at(i))
        //});
        //println!();

        let mut c_dft: VecZnxDft = module.new_vec_znx_dft(cols);
        module.vmp_apply_dft(&mut c_dft, &vec_have, &vmp_pmat, &mut buf);

        let mut c_big: VecZnxBig = c_dft.as_vec_znx_big();
        module.vec_znx_idft_tmp_a(&mut c_big, &mut c_dft, cols);

        let mut res: VecZnx = module.new_vec_znx(cols);
        module.vec_znx_big_normalize(log_base2k, &mut res, &c_big, &mut buf);

        //println!("OUTPUT");
        //(0..res.limbs()).for_each(|i|{
        //    println!("{}: {:?}", i, res.at(i))
        //});
        //println!();

        let mut vec_want: VecZnx = module.new_vec_znx(limbs);
        vec_want.encode_coeff_i64(log_base2k, log_k, value << log_gap_out, 1, 2);

        let mut have: Vec<i64> = vec![i64::default(); module.n()];
        let mut want: Vec<i64> = vec![i64::default(); module.n()];

        res.decode_vec_i64(log_base2k, log_k, &mut have);
        vec_want.decode_vec_i64(log_base2k, log_k, &mut want);

        //println!("{:?}", want);
        //println!("{:?}", have);

        izip!(want, have).for_each(|(a, b)| assert_eq!(a, b));
    });
}
