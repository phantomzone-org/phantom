use crate::address::Address;
use crate::decompose::Decomposer;
use crate::gadget::Gadget;
use crate::parameters::GADGETDECOMP;
use crate::trace::{a_apply_trace_into_a, a_apply_trace_into_b};
use math::automorphism::AutoPermMap;
use math::modulus::montgomery::Montgomery;
use math::modulus::{WordOps, ONCE};
use math::poly::Poly;
use math::ring::Ring;

pub struct CircuitBootstrapper {
    pub test_vectors: Vec<Poly<Montgomery<u64>>>,
    pub log_gap: usize,
    pub log_base: usize,
}

impl CircuitBootstrapper {
    pub fn new(ring: &Ring<u64>, log_n_lwe: usize, log_base: usize) -> Self {
        let log_gap: usize = ring.log_n() - log_n_lwe;

        let mut test_vectors: Vec<Poly<u64>> = Vec::new();

        let log_q: usize = ring.modulus.q.log2() as _;

        let d: usize = (log_q + log_base - 1) / log_base;

        let drift: i32 = 1 << (log_gap - 1);

        let n: usize = ring.n();

        (0..d).for_each(|i| {
            let mut poly: Poly<Montgomery<u64>> = ring.new_poly();

            let w: u64 = 1 << ((i * log_base) as u64);

            let one: Montgomery<u64> = ring.modulus.montgomery.prepare::<ONCE>(w);
            let minus_one: Montgomery<u64> = ring.modulus.q() - one;

            (-drift..drift).for_each(|j| {
                if j < 0 {
                    poly.0[n - (-j as usize)] = minus_one;
                } else {
                    poly.0[j as usize] = one;
                }
            });

            ring.ntt_inplace::<false>(&mut poly);
            test_vectors.push(poly)
        });

        Self {
            test_vectors: test_vectors,
            log_gap: log_gap,
            log_base: log_base,
        }
    }

    pub fn bootstrap_to_address(
        &self,
        ring_pbs: &Ring<u64>,
        ring_lwe: &Ring<u64>,
        value: usize,
        max_address: usize,
        address: &mut Address,
    ) {

        //let mut decomposer: Decomposer = Decomposer::new(ring_pbs, SUBDECOMP);

        //let base: Vec<usize> = Vec::new();

        //

        //let address_decomposed: Vec<u64> = decomposer.decompose(ring_pbs, value as u32, base);
    }

    pub fn circuit_bootstrap(
        &self,
        ring: &Ring<u64>,
        value: usize,
        buf0: &mut Poly<u64>,
        buf1: &mut Poly<u64>,
        buf2: &mut Poly<u64>,
        a: &mut Gadget<Poly<u64>>,
    ) {
        buf0.zero();

        let n: usize = ring.n();
        assert!(
            value <= 2 * n,
            "invalid argument: value={} > 2*n={}",
            value,
            n
        );

        if value < n {
            buf0.0[value] = 1;
        } else {
            buf0.0[value - n] = ring.modulus.q() - 1;
        }

        ring.ntt_inplace::<true>(buf0);

        self.test_vectors
            .iter()
            .enumerate()
            .for_each(|(i, test_vector)| {
                ring.a_mul_b_montgomery_into_c::<ONCE>(test_vector, buf0, buf1);
                ring.switch_degree::<true>(buf1, buf2, a.at_mut(i));
            });
    }

    pub fn post_process(
        &self,
        ring: &Ring<u64>,
        log_gap_in: usize,
        log_gap_out: usize,
        trace_gal_els: &[usize],
        auto_perms: &AutoPermMap,
        buf: &mut [Poly<u64>; 6],
        a: &mut Gadget<Poly<u64>>,
    ) {
        a.value.iter_mut().for_each(|ai| {
            self.post_process_core(
                ring,
                log_gap_in,
                log_gap_out,
                trace_gal_els,
                auto_perms,
                buf,
                ai,
            );
        })
    }

    fn post_process_core(
        &self,
        ring: &Ring<u64>,
        log_gap_in: usize,
        log_gap_out: usize,
        trace_gal_els: &[usize],
        auto_perms: &AutoPermMap,
        buf: &mut [Poly<u64>; 6],
        a: &mut Poly<u64>,
    ) {
        let step_start: usize = ring.log_n() - log_gap_in;
        let step_end = ring.log_n();

        let [buf0, buf1, buf2, buf3, buf4, buf5] = buf;

        // First partial trace, vanishes all coefficients which are not multiples of gap_in
        // [1, 1, 1, 1, 0, 0, 0, ..., 0, 0, -1, -1, -1, -1] -> [1, 0, 0, 0, 0, 0, 0, ..., 0, 0, 0, 0, 0, 0]
        a_apply_trace_into_a::<false, true>(
            ring,
            step_start,
            step_end,
            trace_gal_els,
            auto_perms,
            buf0,
            buf1,
            a,
        );

        // If gap_out < gap_in, then we need to repack, i.e. reduce the cap between
        // coefficients.
        if log_gap_in > log_gap_out {
            let step_end: usize = step_start;
            let step_start: usize = 0;
            let steps: usize = 1 << (ring.log_n() - log_gap_in);

            // Cyclic shift by X^{-gap_in}
            let x_pow_in: &mut Poly<u64> = buf4;
            x_pow_in.zero();
            x_pow_in.0[ring.n() - (1 << log_gap_in)] = ring.modulus.montgomery.minus_one();
            ring.ntt_inplace::<true>(x_pow_in);

            // Cyclic shift by X^{-gap_out}
            let x_pow_out: &mut Poly<u64> = buf5;
            x_pow_out.zero();
            x_pow_out.0[ring.n() - (1 << log_gap_out)] = ring.modulus.montgomery.minus_one();
            ring.ntt_inplace::<true>(x_pow_out);

            buf3.zero();

            // For each coefficients that can be packed, i.e. n / gap_in
            (0..steps).for_each(|i: usize| {
                // Cyclic shift the input and output by their respective X^{-gap_in} and X^{-gap_out}
                if i != 0 {
                    ring.a_mul_b_montgomery_into_a::<ONCE>(x_pow_in, a);
                    ring.a_mul_b_montgomery_into_a::<ONCE>(x_pow_out, buf3);
                }

                // Trace(x * X^{-gap_in}): extracts the X^{gap_in}th coefficient
                a_apply_trace_into_b::<false, true>(
                    ring,
                    step_start,
                    step_end,
                    trace_gal_els,
                    auto_perms,
                    a,
                    buf0,
                    buf1,
                    buf2,
                );

                // Aggregates on the output which gets shifted by X^{-gap_out}
                if i != steps - 1 {
                    ring.a_add_b_into_b::<ONCE>(buf2, buf3);
                } else {
                    ring.a_add_b_into_c::<ONCE>(buf2, buf3, a);
                }
            });

            // Cyclic shift the output back to its original position
            x_pow_out.zero();
            x_pow_out.0[(1 << log_gap_out) * (steps - 1)] = ring.modulus.montgomery.one();
            ring.ntt_inplace::<true>(x_pow_out);
            ring.a_mul_b_montgomery_into_a::<ONCE>(x_pow_out, a);
        }
    }
}
