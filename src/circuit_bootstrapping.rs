use crate::address::Address;
use crate::trace::{trace, trace_inplace};
use base2k::{Infos, Module, VecZnx, VecZnxOps, VmpPMat, VmpPMatOps};
use itertools::izip;

pub struct CircuitBootstrapper {
    pub test_vectors: Vec<VecZnx>,
    pub log_gap: usize,
    pub log_base2k: usize,
}

impl CircuitBootstrapper {
    pub fn new(module: &Module, log_n_lwe: usize, log_base2k: usize, limbs: usize) -> Self {
        let log_gap: usize = module.log_n() - log_n_lwe;

        let mut test_vectors: Vec<VecZnx> = Vec::new();

        let drift: usize = 1 << (log_gap - 1);

        let n: usize = module.n();

        let mut ones: Vec<i64> = vec![i64::default(); module.n()];
        ones[0] = 1;
        (1..drift).for_each(|i| {
            ones[i] = 1;
            ones[n - i] = -1;
        });

        (0..limbs).for_each(|i| {
            let mut tv: VecZnx = module.new_vec_znx(limbs * log_base2k);
            tv.at_mut(i).copy_from_slice(&ones);
            test_vectors.push(tv)
        });

        Self {
            test_vectors: test_vectors,
            log_gap: log_gap,
            log_base2k: log_base2k,
        }
    }

    pub fn bootstrap_to_address(
        &self,
        module_pbs: &Module,
        module_lwe: &Module,
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
        module: &Module,
        value: i64,
        buf_pbs: &mut VecZnx,
        a: &mut Vec<VecZnx>,
    ) {
        let n: usize = module.n();
        assert!(
            value < n as i64,
            "invalid argument: value={} > n={}",
            value,
            n
        );

        assert!(
            value > -(n as i64),
            "invalid argument: value={} < -n={}",
            value,
            -(n as i64)
        );

        assert!(
            a.len() <= self.test_vectors.len(),
            "invalid argument a: a.len()={} > self.test_vectors.len()={}",
            a.len(),
            self.test_vectors.len()
        );

        izip!(a.iter_mut(), self.test_vectors.iter()).for_each(|(ai, ti)| {
            module.vec_znx_rotate(value, buf_pbs, ti);
            buf_pbs.switch_degree(ai);
        });
    }

    pub fn post_process(
        &self,
        module: &Module,
        log_gap_in: usize,
        log_gap_out: usize,
        b: &mut VmpPMat,
        a: &mut Vec<VecZnx>,
        buf: &mut [VecZnx; 4],
    ) {
        a.iter_mut().for_each(|ai| {
            self.post_process_core(module, log_gap_in, log_gap_out, ai, buf);
        });

        let mut buf: Vec<u8> =
            vec![u8::default(); module.vmp_prepare_contiguous_tmp_bytes(b.rows(), b.cols())];
        module.vmp_prepare_dblptr(b, a, &mut buf)
    }

    fn post_process_core(
        &self,
        module: &Module,
        log_gap_in: usize,
        log_gap_out: usize,
        a: &mut VecZnx,
        buf: &mut [VecZnx; 4],
    ) {
        let step_start: usize = module.log_n() - log_gap_in;
        let step_end = module.log_n();

        let [buf0, buf1, buf2, buf3] = buf;
        let mut carry = vec![u8::default(); module.n() << 3];

        // First partial trace, vanishes all coefficients which are not multiples of gap_in
        // [1, 1, 1, 1, 0, 0, 0, ..., 0, 0, -1, -1, -1, -1] -> [1, 0, 0, 0, 0, 0, 0, ..., 0, 0, 0, 0, 0, 0]
        trace_inplace::<false>(
            module,
            self.log_base2k,
            step_start,
            step_end,
            a,
            Some(buf1),
            buf0,
            &mut carry,
        );

        // If gap_out < gap_in, then we need to repack, i.e. reduce the cap between
        // coefficients.
        if log_gap_in > log_gap_out {
            let step_start: usize = 0;
            let step_end: usize = step_start;
            let steps: usize = 1 << (module.log_n() - log_gap_in);

            // For each coefficients that can be packed, i.e. n / gap_in
            (0..steps).for_each(|i: usize| {
                // Cyclic shift the input and output by their respective X^{-gap_in} and X^{-gap_out}
                if i != 0 {
                    module.vec_znx_rotate_inplace(-(1 << log_gap_in), a);
                    module.vec_znx_rotate_inplace(-(1 << log_gap_out), buf3);
                }

                // Trace(x * X^{-gap_in}): extracts the X^{gap_in}th coefficient
                trace::<false>(
                    module,
                    self.log_base2k,
                    step_start,
                    step_end,
                    a,
                    buf0,
                    buf1,
                    &mut carry,
                );

                // Aggregates on the output which gets shifted by X^{-gap_out}
                if i != steps - 1 {
                    module.vec_znx_add_inplace(buf3, buf2);
                } else {
                    module.vec_znx_add(a, buf2, buf3);
                }
            });

            a.normalize(self.log_base2k, &mut carry);

            // Cyclic shift the output back to its original position
            module.vec_znx_rotate_inplace((1 << log_gap_out) * (steps - 1) as i64, a);
        }
    }
}
