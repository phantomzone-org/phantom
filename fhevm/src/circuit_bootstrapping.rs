use crate::address::Address;
use crate::decompose::{Decomposer, Precomp};
use crate::trace::{trace, trace_inplace, trace_tmp_bytes};
use base2k::{
    alloc_aligned, switch_degree, Infos, Module, VecZnx, VecZnxOps, VecZnxVec, VmpPMatOps,
};
use itertools::izip;
use std::cmp::min;

pub struct CircuitBootstrapper {
    pub log_base2k: usize,
    pub cols: usize,
}

impl CircuitBootstrapper {
    pub fn new(log_base2k: usize, cols: usize) -> Self {
        Self {
            cols: cols,
            log_base2k: log_base2k,
        }
    }

    fn gen_test_vector(
        &self,
        module: &Module,
        log_max_value: usize,
        cols: usize,
    ) -> (Vec<VecZnx>, usize) {
        let log_n: usize = module.log_n();

        assert!(
            log_max_value <= log_n - 1,
            "invalid argument: log_max_value={} > log_n={}",
            log_max_value,
            log_n
        );

        let log_max_drift: usize = log_n - log_max_value - 1;
        let max_drift: usize = 1 << log_max_drift;
        let n: usize = 1 << log_n;
        let mut test_vectors: Vec<VecZnx> = Vec::new();

        let mut ones: Vec<i64> = alloc_aligned::<i64>(module.n());
        ones[0] = 1;
        (1..max_drift).for_each(|i| {
            ones[i] = 1;
            ones[n - i] = -1;
        });

        (0..cols).for_each(|i| {
            let mut tv: VecZnx = module.new_vec_znx(self.cols);
            tv.at_mut(i).copy_from_slice(&ones);
            test_vectors.push(tv)
        });

        (test_vectors, log_max_drift)
    }

    /// bootstraps `value` to `Address`
    ///
    /// # Arguments
    ///
    /// * `module_pbs`: module for programmable bootstrapping
    /// * `module_lwe`: module for address-level ops
    /// * `value`: value to bootstrap to address
    /// * `address`: address receiver
    /// * `buf_pbs`: [VecZnx] buffer of degree `module_pbs.n()`
    ///
    /// Adds an offset to an address
    ///
    /// # Arguments
    ///
    /// * `module_pbs`: module for the programmable bootstrapping.
    /// * `module_lwe`: module of the [Address].
    /// * `offset`: value to add to the [Address].
    /// * `max_address`: maximum value of the [Address].
    /// * `address`: [Address] on which to add the offset.
    /// * `buf_pbs`: [VecZnx] buffer of degree `module_pbs.n()`.
    pub fn bootstrap_to_address(
        &self,
        module_pbs: &Module,
        module_lwe: &Module,
        decomposer: &mut Decomposer,
        precomp: &Precomp,
        value: u32,
        address: &mut Address,
        tmp_bytes: &mut [u8],
    ) {
        debug_assert_eq!(precomp.base_1d, address.base_2d.as_1d());

        let value_wrap: u32 = value % address.max() as u32;

        // 3) LWE -> [LWE, LWE, LWE, ...]
        let addr_decomp: Vec<u8> = decomposer.decompose(&module_pbs, precomp, value_wrap);

        debug_assert_eq!(
            precomp.recomp(&addr_decomp),
            value_wrap,
            "{} != {}",
            precomp.recomp(&addr_decomp),
            value_wrap
        );

        // buf RGSW
        let mut buf_addr: Vec<VecZnx> = Vec::new();
        (0..address.rows).for_each(|_| buf_addr.push(module_lwe.new_vec_znx(address.cols)));

        let mut buf: Vec<u8> =
            alloc_aligned(module_lwe.vmp_prepare_tmp_bytes(address.rows(), address.cols()));

        let mut i: usize = 0;

        (0..address.n2()).for_each(|hi| {
            let mut sum_base: u8 = 0;

            (0..address.n1(hi)).for_each(|lo: usize| {
                let base: u8 = address.base_2d.0[hi].0[lo];

                // 4) LWE[i] -> RGSW
                let log_gap_in: usize = self.circuit_bootstrap(
                    module_pbs,
                    addr_decomp[i],
                    base as usize,
                    &mut buf_addr,
                    tmp_bytes,
                );

                self.post_process(
                    module_lwe,
                    log_gap_in as u8,
                    sum_base,
                    (1 << base) - 1,
                    &mut buf_addr,
                    tmp_bytes,
                );

                /*
                println!("log_gap_in: {}", log_gap_in);
                println!("value: {}", addr_decomp[i]);
                let mut values: Vec<i64> = vec![0; module_lwe.n()];
                let mut j: usize = 0;
                buf_addr.iter_mut().for_each(|buf_addr| {
                    buf_addr.decode_vec_i64(
                        self.log_base2k,
                        self.log_base2k * (buf_addr.cols() - 1),
                        &mut values,
                    );
                    println!("{}: {:?}", j, &values[..32]);
                    j += 1;
                });
                 */

                module_lwe.vmp_prepare_dblptr(
                    &mut address.coordinates_rsh[hi].value[lo],
                    &buf_addr.dblptr(),
                    &mut buf,
                );

                buf_addr.iter_mut().for_each(|buf_addr_i| {
                    module_lwe.vec_znx_automorphism_inplace(-1, buf_addr_i, buf_addr_i.cols());
                });

                module_lwe.vmp_prepare_dblptr(
                    &mut address.coordinates_lsh[hi].value[lo],
                    &buf_addr.dblptr(),
                    &mut buf,
                );

                i += 1;
                sum_base += base;
            })
        });

        debug_assert_eq!(
            address.debug_as_u32(module_lwe),
            value_wrap,
            "address: {} != value_wrap: {}",
            address.debug_as_u32(module_lwe),
            value_wrap
        );
    }

    pub fn circuit_bootstrap(
        &self,
        module: &Module,
        value: u8,
        log_max_val: usize,
        a: &mut Vec<VecZnx>,
        tmp_bytes: &mut [u8],
    ) -> usize {
        let n: usize = module.n();

        let (test_vectors, log_max_drift): (Vec<VecZnx>, usize) =
            self.gen_test_vector(module, log_max_val, a.len());

        assert!(
            a.len() <= test_vectors.len(),
            "invalid argument a: a.len()={} > self.test_vectors.len()={}",
            a.len(),
            test_vectors.len()
        );

        assert!(
            tmp_bytes.len() >= circuit_bootstrap_tmp_bytes(module, self.cols),
            "invalid tmp_bytes: tmp_bytes.len()={} < circuit_bootstrap_tmp_bytes={}",
            tmp_bytes.len(),
            circuit_bootstrap_tmp_bytes(module, self.cols)
        );

        let mut buf_pbs: VecZnx = VecZnx::from_bytes_borrow(n, self.cols, tmp_bytes);

        izip!(a.iter_mut(), test_vectors.iter()).for_each(|(ai, ti)| {
            module.vec_znx_rotate(
                (value as i64) * (1 << (log_max_drift + 1)) as i64,
                &mut buf_pbs,
                ti,
            );
            switch_degree(ai, &buf_pbs);
        });

        (log_max_drift + 1) - (module.log_n() - a[0].log_n())
    }

    pub fn post_process(
        &self,
        module: &Module,
        log_gap_in: u8,
        log_gap_out: u8,
        max_value: usize,
        a: &mut Vec<VecZnx>,
        tmp_byte: &mut [u8],
    ) {
        a.iter_mut().for_each(|ai| {
            self.post_process_core(module, log_gap_in, log_gap_out, max_value, ai, tmp_byte);
        });
    }

    fn post_process_core(
        &self,
        module: &Module,
        log_gap_in: u8,
        log_gap_out: u8,
        max_value: usize,
        a: &mut VecZnx,
        tmp_bytes: &mut [u8],
    ) {
        assert!(
            tmp_bytes.len() >= post_process_tmp_bytes(module, a.cols()),
            "invalid tmp_bytes: tmp_bytes.len() < post_process_tmp_bytes"
        );

        let n: usize = module.n();
        let cols: usize = a.cols();

        let step_start: usize = module.log_n() - log_gap_in as usize + 1;
        let step_end = module.log_n();

        let bytes_of_vec_znx = module.bytes_of_vec_znx(cols);

        let (tmp_bytes_buf2, tmp_bytes) = tmp_bytes.split_at_mut(bytes_of_vec_znx);
        let (tmp_bytes_buf3, trace_tmp_bytes) = tmp_bytes.split_at_mut(bytes_of_vec_znx);

        let mut buf0: VecZnx = VecZnx::from_bytes_borrow(n, cols, tmp_bytes_buf2);
        let mut buf1: VecZnx = VecZnx::from_bytes_borrow(n, cols, tmp_bytes_buf3);

        // First partial trace, vanishes all coefficients which are not multiples of gap_in
        // [1, 1, 1, 1, 0, 0, 0, ..., 0, 0, -1, -1, -1, -1] -> [1, 0, 0, 0, 0, 0, 0, ..., 0, 0, 0, 0, 0, 0]
        trace_inplace(
            module,
            self.log_base2k,
            step_start,
            step_end,
            a,
            trace_tmp_bytes,
        );

        // If gap_out < gap_in, then we need to repack, i.e. reduce the cap between
        // coefficients.
        if log_gap_in != log_gap_out {
            let step_end: usize = step_start;
            let step_start: usize = 0 as usize;
            let steps: usize = min(max_value + 1, 1 << (module.log_n() - log_gap_in as usize));

            // For each coefficients that can be packed, i.e. n / gap_in
            (0..steps).for_each(|i: usize| {
                // Cyclic shift the input and output by their respective X^{-gap_in} and X^{-gap_out}
                if i != 0 {
                    module.vec_znx_rotate_inplace(-(1 << log_gap_in), a);
                    module.vec_znx_rotate_inplace(-(1 << log_gap_out), &mut buf0);
                }

                // Trace(x * X^{-gap_in}): extracts the X^{gap_in}th coefficient
                if i == 0 {
                    trace(
                        module,
                        self.log_base2k,
                        step_start,
                        step_end,
                        &mut buf0,
                        a,
                        trace_tmp_bytes,
                    );
                } else {
                    trace(
                        module,
                        self.log_base2k,
                        step_start,
                        step_end,
                        &mut buf1,
                        a,
                        trace_tmp_bytes,
                    );
                    module.vec_znx_add_inplace(&mut buf0, &mut buf1);
                }
            });

            a.copy_from(&buf0);

            a.normalize(self.log_base2k, trace_tmp_bytes);

            // Cyclic shift the output back to its original position
            module.vec_znx_rotate_inplace((1 << log_gap_out) * (steps - 1) as i64, a);
        }
    }
}

pub fn bootstrap_to_address_tmp_bytes(
    module_pbs: &Module,
    module_lwe: &Module,
    cols: usize,
) -> usize {
    circuit_bootstrap_tmp_bytes(module_pbs, cols) | post_process_tmp_bytes(module_lwe, cols)
}

pub fn bootstrap_address_tmp_bytes(module_pbs: &Module, module_lwe: &Module, cols: usize) -> usize {
    bootstrap_to_address_tmp_bytes(module_pbs, module_lwe, cols)
}

pub fn post_process_tmp_bytes(module: &Module, cols: usize) -> usize {
    2 * module.bytes_of_vec_znx(cols) + trace_tmp_bytes(module, cols)
}

pub fn circuit_bootstrap_tmp_bytes(module: &Module, cols: usize) -> usize {
    module.bytes_of_vec_znx(cols)
}
