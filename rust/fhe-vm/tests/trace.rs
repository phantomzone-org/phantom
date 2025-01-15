use fhevm::trace::a_apply_trace_into_a;
use math::poly::Poly;
use math::ring::Ring;

#[test]
fn trace_u64() {
    let n: usize = 1 << 5;
    let q_base: u64 = 65537u64;
    let q_power: usize = 1usize;
    let ring: Ring<u64> = Ring::new(n, q_base, q_power);

    sub_test("test_trace::<INV:false, NTT:false>", || {
        test_trace_u64::<false, false>(&ring)
    });
    sub_test("test_trace::<INV:false, NTT:true>", || {
        test_trace_u64::<false, true>(&ring)
    });
    sub_test("test_trace::<INV:true, NTT:false>", || {
        test_trace_u64::<true, false>(&ring)
    });
    sub_test("test_trace::<INV:true, NTT:true>", || {
        test_trace_u64::<true, true>(&ring)
    });
}

fn sub_test<F: FnOnce()>(name: &str, f: F) {
    println!("Running {}", name);
    f();
}

fn test_trace_u64<const INV: bool, const NTT: bool>(ring: &Ring<u64>) {
    let mut poly: Poly<u64> = ring.new_poly();
    let mut buf0: Poly<u64> = ring.new_poly();
    let mut buf1: Poly<u64> = ring.new_poly();

    poly.0
        .iter_mut()
        .enumerate()
        .for_each(|(i, x)| *x = (i + 1) as u64);

    if NTT {
        ring.ntt_inplace::<false>(&mut poly);
    }

    let step_start: usize = 2;

    a_apply_trace_into_a::<INV, NTT>(ring, step_start, &mut buf0, &mut buf1, &mut poly);

    if NTT {
        ring.intt_inplace::<false>(&mut poly);
    }

    let gap: usize = 1 << (ring.log_n() - step_start);

    if INV {
        poly.0.iter().enumerate().for_each(|(i, x)| {
            if i % gap == 0 {
                assert_eq!(*x, 0u64)
            } else {
                assert_eq!(*x, 1 + i as u64)
            }
        });
    } else {
        poly.0.iter().enumerate().for_each(|(i, x)| {
            if i % gap == 0 {
                assert_eq!(*x, 1 + i as u64)
            } else {
                assert_eq!(*x, 0u64)
            }
        });
    }
}
