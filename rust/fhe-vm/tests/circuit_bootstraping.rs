use fhevm::circuit_bootstrapping::Accumulator;
use fhevm::trace::gen_auto_perms;
use math::poly::Poly;
use math::ring::Ring;

#[test]
fn circuit_bootstrapping() {
    let n: usize = 1 << 4;
    let q_base: u64 = 65537;
    let q_power: usize = 1usize;
    let ring_acc: Ring<u64> = Ring::new(n * 2, q_base, q_power);
    let ring: Ring<u64> = Ring::new(n, q_base, q_power);

    let log_gap: usize = 3;
    let log_base: usize = 7;

    let acc: Accumulator = Accumulator::new(&ring_acc, log_gap, log_base);

    let mut buf_acc_0: Poly<u64> = ring_acc.new_poly();
    let mut buf_acc_1: Poly<u64> = ring_acc.new_poly();
    let mut buf_acc_2: Poly<u64> = ring_acc.new_poly();

    let mut a: Vec<Poly<u64>> = Vec::new();

    (0..acc.test_vectors.len()).for_each(|_| a.push(ring.new_poly()));

    let value: usize = 28;

    acc.circuit_bootstrap(
        &ring_acc,
        value,
        &mut buf_acc_0,
        &mut buf_acc_1,
        &mut buf_acc_2,
        &mut a,
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

    println!("log_gap_in: {}", log_gap_in);
    println!("log_gap_out: {}", log_gap_out);

    acc.post_process(
        &ring,
        log_gap_in,
        log_gap_out,
        &trace_gal_els,
        &auto_perms,
        buf,
        &mut a,
    );

    a.iter_mut().for_each(|ai| {
        ring.intt_inplace::<false>(ai);
    });

    println!("{:?}", a[0]);
}
