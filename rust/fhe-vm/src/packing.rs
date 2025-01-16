use crate::trace::gen_auto_perms;
use math::automorphism::{AutoPerm, AutoPermMap};
use math::modulus::barrett::Barrett;
use math::modulus::montgomery::Montgomery;
use math::modulus::ONCE;
use math::poly::Poly;
use math::ring::Ring;
use std::cmp::min;

pub fn pack<const ZEROGARBAGE: bool, const NTT: bool>(
    ring: &Ring<u64>,
    polys: &mut Vec<Option<&mut Poly<u64>>>,
    pack_galois_elements: &[usize],
    auto_perms: &AutoPermMap,
    log_gap: usize,
) {
    let log_n: usize = ring.log_n();
    let log_start: usize = log_n - log_gap;
    let mut log_end: usize = log_n;

    let mut indices: Vec<usize> = Vec::<usize>::new();

    pack_galois_elements.iter().for_each(|gal_el| {
        if let Some(auto_perm) = auto_perms.get(gal_el) {
            assert!(
                auto_perm.ntt == true,
                "invalid AutoPerm NTT flag for gal_el={}, expected to be true but is false",
                gal_el
            );
        } else {
            panic!("galois element {} not found in AutoPermMap", gal_el)
        }
    });

    // Retrives non-empty indexes
    polys.iter().enumerate().for_each(|(i, poly)| {
        if !poly.is_none() {
            indices.push(i);
        }
    });

    let gap: usize = max_gap(&indices);

    if !ZEROGARBAGE {
        if gap > 0 {
            log_end -= gap.trailing_zeros() as usize;
        }
    }

    assert!(
        log_start < log_end,
        "invalid input polys: gap between non None value is smaller than 2^log_gap"
    );

    let n_inv: Barrett<u64> = ring
        .modulus
        .barrett
        .prepare(ring.modulus.inv(1 << (log_end - log_start)));

    indices.iter().for_each(|i| {
        if let Some(poly) = polys[*i].as_deref_mut() {
            if !NTT {
                ring.ntt_inplace::<true>(poly);
            }
            ring.a_mul_b_scalar_barrett_into_a::<ONCE>(&n_inv, poly);
        }
    });

    let x_pow2: Vec<Poly<Montgomery<u64>>> = ring.gen_x_pow_2::<true, false>(log_n);
    let mut tmp: Poly<u64> = ring.new_poly();

    for i in log_start..log_end {
        let t: usize = 1 << (log_n - 1 - i);

        let (polys_lo, polys_hi) = polys.split_at_mut(t);

        let gal_el = pack_galois_elements[i];

        for j in 0..t {
            if let Some(poly_hi) = polys_hi[j].as_mut() {
                ring.a_mul_b_montgomery_into_a::<ONCE>(&x_pow2[log_n - i - 1], poly_hi);

                if let Some(poly_lo) = polys_lo[j].as_mut() {
                    ring.a_sub_b_into_c::<1, ONCE>(poly_lo, poly_hi, &mut tmp);
                    ring.a_add_b_into_b::<ONCE>(poly_hi, poly_lo);
                }
            }

            if let Some(poly_lo) = polys_lo[j].as_mut() {
                if !polys_hi[j].is_none() {
                    if let Some(auto_perm) = auto_perms.get(&gal_el) {
                        ring.a_apply_automorphism_from_perm_add_b_into_b::<ONCE, true>(
                            &tmp, auto_perm, poly_lo,
                        );
                    } else {
                        panic!("galois element {} not found in AutoPermMap", gal_el)
                    }
                } else {
                    if let Some(auto_perm) = auto_perms.get(&gal_el) {
                        ring.a_apply_automorphism_from_perm_into_b::<true>(
                            poly_lo, auto_perm, &mut tmp,
                        );
                        ring.a_add_b_into_b::<ONCE>(&tmp, poly_lo);
                    } else {
                        panic!("galois element {} not found in AutoPermMap", gal_el)
                    }
                }
            } else if let Some(poly_hi) = polys_hi[j].as_mut() {
                if let Some(auto_perm) = auto_perms.get(&gal_el) {
                    ring.a_apply_automorphism_from_perm_into_b::<true>(
                        poly_hi, auto_perm, &mut tmp,
                    );
                    ring.a_sub_b_into_a::<1, ONCE>(&tmp, poly_hi);
                    std::mem::swap(&mut polys_lo[j], &mut polys_hi[j]);
                } else {
                    panic!("galois element {} not found in AutoPermMap", gal_el)
                }
            }
        }

        polys.truncate(t);
    }

    if !NTT {
        if let Some(poly) = polys[0].as_mut() {
            ring.intt_inplace::<false>(poly);
        }
    }
}

// Returns the largest gap between two values in an ordered array of distinct values.
// Panics if the array is not ordered or values are not distincts.
fn max_gap(vec: &[usize]) -> usize {
    let mut gap: usize = usize::MAX;
    for i in 1..vec.len() {
        let (l, r) = (vec[i - 1], vec[i]);
        assert!(
            r > l,
            "invalid input vec: not sorted or collision between indices"
        );
        gap = min(gap, r - l);
        if gap == 1 {
            break;
        }
    }
    gap
}

pub struct StreamRepacker {
    accumulators: Vec<Accumulator>,
    tmp_a: Poly<u64>,
    tmp_b: Poly<u64>,
    x_pow_2: Vec<Poly<Montgomery<u64>>>,
    gal_els: Vec<usize>,
    auto_perms: AutoPermMap,
    n_inv: Barrett<u64>,
    counter: usize,
}

pub struct Accumulator {
    buf: Poly<u64>,
    value: bool,
    control: bool,
}

impl Accumulator {
    pub fn new(r: &Ring<u64>) -> Self {
        Self {
            buf: r.new_poly(),
            value: false,
            control: false,
        }
    }
}

impl StreamRepacker {
    pub fn new(ring: &Ring<u64>) -> Self {
        let mut accumulators: Vec<Accumulator> = Vec::<Accumulator>::new();

        let log_n: usize = ring.log_n();

        (0..log_n).for_each(|_| accumulators.push(Accumulator::new(ring)));

        let (auto_perms, gal_els) = gen_auto_perms::<true>(ring);

        Self {
            accumulators: accumulators,
            tmp_a: ring.new_poly(),
            tmp_b: ring.new_poly(),
            x_pow_2: ring.gen_x_pow_2::<true, false>(log_n),
            gal_els: gal_els,
            auto_perms: auto_perms,
            n_inv: ring
                .modulus
                .barrett
                .prepare(ring.modulus.inv(1 << log_n as u64)),
            counter: 0,
        }
    }

    pub fn reset(&mut self) {
        for i in 0..self.accumulators.len() {
            self.accumulators[i].value = false;
            self.accumulators[i].control = false;
        }
        self.counter = 0;
    }

    pub fn add<const NTT: bool>(
        &mut self,
        r: &Ring<u64>,
        a: Option<&Poly<u64>>,
        result: &mut Vec<Poly<u64>>,
    ) {
        assert!(NTT, "invalid parameterization: const NTT must be true");
        pack_core::<NTT>(
            r,
            a,
            &mut self.accumulators,
            &self.n_inv,
            &self.x_pow_2,
            &self.gal_els,
            &self.auto_perms,
            &mut self.tmp_a,
            &mut self.tmp_b,
            0,
        );
        self.counter += 1;
        if self.counter == r.n() {
            result.push(self.accumulators[r.log_n() - 1].buf.clone());
            self.reset();
        }
    }

    pub fn flush<const NTT: bool>(&mut self, r: &Ring<u64>, result: &mut Vec<Poly<u64>>) {
        assert!(NTT, "invalid parameterization: const NTT must be true");
        if self.counter != 0 {
            while self.counter != r.n() - 1 {
                self.add::<NTT>(r, None, result);
            }
        }
    }
}

fn pack_core<const NTT: bool>(
    r: &Ring<u64>,
    a: Option<&Poly<u64>>,
    accumulators: &mut [Accumulator],
    n_inv: &Barrett<u64>,
    x_pow_2: &[Poly<u64>],
    gal_els: &[usize],
    auto_perms: &AutoPermMap,
    tmp_a: &mut Poly<u64>,
    tmp_b: &mut Poly<u64>,
    i: usize,
) {
    let log_n = r.log_n();

    if i == log_n {
        return;
    }

    let (acc_prev, acc_next) = accumulators.split_at_mut(1);

    if !acc_prev[0].control {
        let acc_mut_ref: &mut Accumulator = &mut acc_prev[0]; // from split_at_mut

        if let Some(a_ref) = a {
            acc_mut_ref.buf.copy_from(a_ref);
            acc_mut_ref.value = true
        } else {
            acc_mut_ref.value = false
        }
        acc_mut_ref.control = true;
    } else {
        if let Some(auto_perm) = auto_perms.get(&gal_els[i]) {
            combine::<true>(
                r,
                &mut acc_prev[0],
                a,
                n_inv,
                &x_pow_2[log_n - i - 1],
                auto_perm,
                tmp_a,
                tmp_b,
                i,
            );
        } else {
            panic!("galois element {} not found in AutoPerms", &gal_els[i])
        }

        acc_prev[0].control = false;

        if acc_prev[0].value {
            pack_core::<NTT>(
                r,
                Some(&acc_prev[0].buf),
                acc_next,
                n_inv,
                x_pow_2,
                gal_els,
                auto_perms,
                tmp_a,
                tmp_b,
                i + 1,
            );
        } else {
            pack_core::<NTT>(
                r,
                None,
                acc_next,
                n_inv,
                x_pow_2,
                gal_els,
                auto_perms,
                tmp_a,
                tmp_b,
                i + 1,
            );
        }
    }
}

fn combine<const NTT: bool>(
    r: &Ring<u64>,
    acc: &mut Accumulator,
    b: Option<&Poly<u64>>,
    n_inv: &Barrett<u64>,
    x_pow_2: &Poly<u64>,
    auto_perm: &AutoPerm,
    tmp_a: &mut Poly<u64>,
    tmp_b: &mut Poly<u64>,
    i: usize,
) {
    let a: &mut Poly<u64> = &mut acc.buf;

    if acc.value {
        if i == 0 {
            r.a_mul_b_scalar_barrett_into_a::<ONCE>(n_inv, a);
        }

        if let Some(b) = b {
            // tmp_a = b * X^t
            r.a_mul_b_montgomery_into_c::<ONCE>(b, x_pow_2, tmp_a);

            if i == 0 {
                r.a_mul_b_scalar_barrett_into_a::<ONCE>(&n_inv, tmp_a);
            }

            // tmp_b = a - b*X^t
            r.a_sub_b_into_c::<1, ONCE>(a, tmp_a, tmp_b);

            // a = a + b * X^t
            r.a_add_b_into_b::<ONCE>(tmp_a, a);

            // a = a + b * X^t + phi(a - b * X^t)
            r.a_apply_automorphism_from_perm_add_b_into_b::<ONCE, NTT>(tmp_b, auto_perm, a);
        } else {
            // tmp_a = phi(a)
            r.a_apply_automorphism_from_perm_into_b::<NTT>(a, auto_perm, tmp_a);
            // a = a + phi(a)
            r.a_add_b_into_b::<ONCE>(tmp_a, a);
        }
    } else {
        if let Some(b) = b {
            // tmp_b = b * X^t
            r.a_mul_b_montgomery_into_c::<ONCE>(b, x_pow_2, tmp_b);

            if i == 0 {
                r.a_mul_b_scalar_barrett_into_a::<ONCE>(&n_inv, tmp_b);
            }

            // tmp_a = phi(b * X^t)
            r.a_apply_automorphism_from_perm_into_b::<NTT>(tmp_b, auto_perm, tmp_a);

            // a = (b* X^t - phi(b* X^t))
            r.a_sub_b_into_c::<1, ONCE>(tmp_b, tmp_a, a);
            acc.value = true
        }
    }
}
