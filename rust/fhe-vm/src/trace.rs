use math::automorphism::AutoPermMap;
use math::modulus::barrett::Barrett;
use math::modulus::ONCE;
use math::poly::Poly;
use math::ring::Ring;

pub fn gen_auto_perms<const NTT: bool>(ring: &Ring<u64>) -> (AutoPermMap, Vec<usize>) {
    let mut auto_perms = AutoPermMap::new();
    let mut gal_els: Vec<usize> = Vec::new();
    (0..ring.log_n()).for_each(|i: usize| {
        gal_els.push(auto_perms.gen::<_, NTT>(ring, (1 << i) >> 1, i == 0));
    });
    (auto_perms, gal_els)
}

pub fn a_apply_trace_into_a<const INV: bool, const NTT: bool>(
    ring: &Ring<u64>,
    step_start: usize,
    step_end: usize,
    trace_gal_els: &[usize],
    auto_perms: &AutoPermMap,
    buf0: &mut Poly<u64>,
    buf1: &mut Poly<u64>,
    a: &mut Poly<u64>,
) {
    assert!(
        step_start <= step_end,
        "invalid argument step_start: step_start={} > step_end={}",
        step_start,
        step_end
    );

    assert!(
        step_end <= ring.log_n(),
        "invalid argument step_end: step_end={} > self.log_n()={}",
        step_end,
        ring.log_n()
    );

    let log_steps: usize = step_end - step_start;
    if log_steps > 0 {
        let n_inv: Barrett<u64> = ring
            .modulus
            .barrett
            .prepare(ring.modulus.inv(1 << log_steps));
        if INV {
            ring.a_mul_b_scalar_barrett_into_c::<ONCE>(&n_inv, a, buf1);
            trace_core::<NTT>(
                ring,
                step_start,
                step_end,
                trace_gal_els,
                auto_perms,
                buf0,
                buf1,
            );
            ring.a_sub_b_into_a::<1, ONCE>(buf1, a);
        } else {
            ring.a_mul_b_scalar_barrett_into_a::<ONCE>(&n_inv, a);
            trace_core::<NTT>(
                ring,
                step_start,
                step_end,
                trace_gal_els,
                auto_perms,
                buf0,
                a,
            );
        }
    }
}

pub fn a_apply_trace_into_b<const INV: bool, const NTT: bool>(
    ring: &Ring<u64>,
    step_start: usize,
    step_end: usize,
    trace_gal_els: &[usize],
    auto_perms: &AutoPermMap,
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

    assert!(
        step_start <= step_end,
        "invalid argument step_start: step_start={} > step_end={}",
        step_start,
        step_end
    );

    assert!(
        step_end <= ring.log_n(),
        "invalid argument step_end: step_end={} > self.log_n()={}",
        step_end,
        ring.log_n()
    );

    let log_steps: usize = step_end - step_start;
    if log_steps > 0 {
        let n_inv: Barrett<u64> = ring
            .modulus
            .barrett
            .prepare(ring.modulus.inv(1 << log_steps));
        if INV {
            ring.a_mul_b_scalar_barrett_into_c::<ONCE>(&n_inv, a, buf1);
            trace_core::<NTT>(
                ring,
                step_start,
                step_end,
                trace_gal_els,
                auto_perms,
                buf0,
                buf1,
            );
            ring.a_sub_b_into_a::<1, ONCE>(buf1, b);
        } else {
            ring.a_mul_b_scalar_barrett_into_c::<ONCE>(&n_inv, a, b);
            trace_core::<NTT>(
                ring,
                step_start,
                step_end,
                trace_gal_els,
                auto_perms,
                buf0,
                b,
            );
        }
    } else {
        b.copy_from(a);
    }
}

fn trace_core<const NTT: bool>(
    ring: &Ring<u64>,
    step_start: usize,
    step_end: usize,
    trace_gal_els: &[usize],
    auto_perms: &AutoPermMap,
    buf: &mut Poly<u64>,
    a: &mut Poly<u64>,
) {
    (step_start..step_end).for_each(|i| {
        if let Some(auto_perm) = auto_perms.get(&trace_gal_els[i]) {
            ring.a_apply_automorphism_from_perm_into_b::<NTT>(a, auto_perm, buf);
            ring.a_add_b_into_b::<ONCE>(buf, a);
        } else {
            panic!(
                "galois element {} not found in AutoPermMap",
                trace_gal_els[i]
            )
        }
    });
}
