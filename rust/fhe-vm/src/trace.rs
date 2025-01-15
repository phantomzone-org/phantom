use math::modulus::barrett::Barrett;
use math::modulus::ONCE;
use math::poly::Poly;
use math::ring::Ring;

pub fn a_apply_trace_into_a<const INV: bool, const NTT: bool>(
    ring: &Ring<u64>,
    step_start: usize,
    buf0: &mut Poly<u64>,
    buf1: &mut Poly<u64>,
    a: &mut Poly<u64>,
) {
    assert!(
        step_start <= ring.log_n(),
        "invalid argument step_start: step_start={} > self.log_n()={}",
        step_start,
        ring.log_n()
    );

    let log_steps: usize = ring.log_n() - step_start;
    if log_steps > 0 {
        let n_inv: Barrett<u64> = ring
            .modulus
            .barrett
            .prepare(ring.modulus.inv(1 << log_steps));
        if INV {
            ring.a_mul_b_scalar_barrett_into_c::<ONCE>(&n_inv, a, buf1);
            trace_core::<NTT>(ring, step_start, buf0, buf1);
            ring.a_sub_b_into_a::<1, ONCE>(buf1, a);
        } else {
            ring.a_mul_b_scalar_barrett_into_a::<ONCE>(&n_inv, a);
            trace_core::<NTT>(ring, step_start, buf0, a);
        }
    }
}

pub fn a_apply_trace_into_b<const INV: bool, const NTT: bool>(
    ring: &Ring<u64>,
    step_start: usize,
    a: &Poly<u64>,
    buf0: &mut Poly<u64>,
    buf1: &mut Poly<u64>,
    b: &mut Poly<u64>,
) {
    assert!(
        step_start <= ring.log_n(),
        "invalid argument step_start: step_start={} > ring.log_n()={}",
        step_start,
        ring.log_n()
    );

    let log_steps: usize = ring.log_n() - step_start;
    if log_steps > 0 {
        let n_inv: Barrett<u64> = ring
            .modulus
            .barrett
            .prepare(ring.modulus.inv(1 << log_steps));
        if INV {
            ring.a_mul_b_scalar_barrett_into_c::<ONCE>(&n_inv, a, buf1);
            trace_core::<NTT>(ring, step_start, buf0, buf1);
            ring.a_sub_b_into_a::<1, ONCE>(buf1, b);
        } else {
            ring.a_mul_b_scalar_barrett_into_c::<ONCE>(&n_inv, a, buf0);
            trace_core::<NTT>(ring, step_start, buf0, b);
        }
    } else {
        b.copy_from(a);
    }
}

fn trace_core<const NTT: bool>(
    ring: &Ring<u64>,
    step_start: usize,
    buf: &mut Poly<u64>,
    a: &mut Poly<u64>,
) {
    let log_nth_root = ring.log_n() + 1;
    let nth_root: usize = 1 << log_nth_root;
    (step_start..ring.log_n()).for_each(|i| {
        let gal_el: usize = ring.galois_element((1 << i) >> 1, i == 0, log_nth_root);

        ring.a_apply_automorphism_into_b::<NTT>(a, gal_el, nth_root, buf);
        ring.a_add_b_into_b::<ONCE>(buf, a);
    });
}
