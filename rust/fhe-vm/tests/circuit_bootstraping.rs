use base2k::{Module, FFT64};
use fhevm::circuit_bootstrapping::CircuitBootstrapper;
use itertools::izip;

#[test]
fn circuit_bootstrapping() {
    let n: usize = 1 << 7;
    let log_q: usize = 54;
    let log_base2k: usize = 16;
    let module: Module = Module::new::<FFT64>(n);

    let module_acc: Module = Module::new::<FFT64>(4 * n);

    let log_gap: usize = 3;
    let log_base: usize = 7;

    //let acc: CircuitBootstrapper = CircuitBootstrapper::new(&ring_acc, ring.log_n(), log_base);
    //acc.init();

    /*
    let mut buf_acc_0: Poly<u64> = ring_acc.new_poly();
    let mut buf_acc_1: Poly<u64> = ring_acc.new_poly();
    let mut buf_acc_2: Poly<u64> = ring_acc.new_poly();

    let mut gadget: Gadget<Poly<u64>> = Gadget::new(&ring, log_base2k, log_q);

    // value in [0, n_acc/2^{log_gap} - 1]
    (0..n_acc / (1 << log_gap)).for_each(|value| {
        // value in [0, n_acc - 2^log_gap]
        let value_scaled: usize = value << log_gap;

        // Maps value in [0, n_acc - 2^log_gap] to X^{value * (2^log_gap*n/n_acc) +/- drift/2^{log_gap-1}}
        acc.circuit_bootstrap(
            &ring_acc,
            value_scaled,
            &mut buf_acc_0,
            &mut buf_acc_1,
            &mut buf_acc_2,
            &mut gadget,
        );

        let buf: &mut [Poly<u64>; 6] = &mut [
            ring.new_poly(),
            ring.new_poly(),
            ring.new_poly(),
            ring.new_poly(),
            ring.new_poly(),
            ring.new_poly(),
        ];

        let (auto_perms, trace_gal_els) = gen_auto_perms::<true>(&ring);

        let log_gap_in: usize = log_gap - (ring_acc.log_n() - ring.log_n());
        let log_gap_out: usize = log_gap_in;

        //println!("log_gap_in: {}", log_gap_in);
        //println!("log_gap_out: {}", log_gap_out);

        // Maps X^(i * 2^{log_gap_in}) to X^(i * 2^{log_gal_out})
        acc.post_process(
            &ring,
            log_gap_in,
            log_gap_out,
            &trace_gal_els,
            &auto_perms,
            buf,
            &mut gadget,
        );

        let mut have: Poly<u64> = ring.new_poly();
        have.0[0] = 1;
        ring.ntt_inplace::<false>(&mut have);

        let [buf0, buf1, buf2, _, _, _] = buf;

        gadget.product_inplace(&ring, buf0, buf1, buf2, &mut have);

        ring.intt_inplace::<false>(&mut have);

        let mut want: Poly<u64> = ring.new_poly();
        want.0[value << log_gap_out] = 1;

        println!("{:?}", want);
        println!("{:?}", have);

        izip!(want.0, have.0).for_each(|(a, b)| assert_eq!(a, b));
    });
    */
}
