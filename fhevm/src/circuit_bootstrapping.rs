use crate::address::Address;
use crate::decompose::{Decomposer, Precomp};
use crate::instructions::r_type::add;
use crate::trace::{trace, trace_inplace, trace_tmp_bytes};
use base2k::{
    alloc_aligned, switch_degree, Infos, Module, VecZnx, VecZnxOps, VecZnxVec, VmpPMatOps,
};
use itertools::izip;
use std::cmp::min;

pub struct CircuitBootstrapper {
    pub test_vectors: Vec<VecZnx>,
    pub log_gap: usize,
    pub log_base2k: usize,
}

impl CircuitBootstrapper {
    pub fn new(module: &Module, log_n_lwe: usize, log_base2k: usize, cols: usize) -> Self {
        let log_gap: usize = module.log_n() - log_n_lwe;

        let mut test_vectors: Vec<VecZnx> = Vec::new();

        let drift: usize = 1 << (log_gap - 1);

        let n: usize = module.n();

        let mut ones: Vec<i64> = alloc_aligned::<i64>(module.n());
        ones[0] = 1;
        (1..drift).for_each(|i| {
            ones[i] = 1;
            ones[n - i] = -1;
        });

        (0..cols).for_each(|i| {
            let mut tv: VecZnx = module.new_vec_znx(cols * log_base2k);
            tv.at_mut(i).copy_from_slice(&ones);
            test_vectors.push(tv)
        });

        Self {
            test_vectors: test_vectors,
            log_gap: log_gap,
            log_base2k: log_base2k,
        }
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
        debug_assert_eq!(precomp.log_bases, address.decomp_flattened());

        // 3) LWE -> [LWE, LWE, LWE, ...]
        let addr_decomp: Vec<i64> = decomposer.decompose(&module_pbs, precomp, value);

        //println!("cols: {}", address.cols);

        // buf RGSW
        let mut buf_addr: Vec<VecZnx> = Vec::new();
        (0..address.rows).for_each(|_| buf_addr.push(module_lwe.new_vec_znx(address.cols)));

        let log_gap: usize = 3;
        let log_gap_in: usize = log_gap - (module_pbs.log_n() - module_lwe.log_n());

        let mut buf: Vec<u8> =
            alloc_aligned(module_lwe.vmp_prepare_tmp_bytes(address.rows(), address.cols()));

        //println!();

        let mut i: usize = 0;
        (0..address.dims_n1()).for_each(|hi| {
            let mut sum_base: u8 = 0;

            (0..address.dims_n2()).for_each(|lo: usize| {
                let base: u8 = address.decomp[hi][lo];

                //println!(": {} log_gap_in: {} log_gap_out: {} value: {}", i, log_gap_in, base * (dims_n_decomp - lo-1), addr_decomp[i]);

                // 4) LWE[i] -> RGSW
                self.circuit_bootstrap(
                    module_pbs,
                    addr_decomp[i] << log_gap,
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
                let mut j: usize = 0;
                buf_addr.iter_mut().for_each(|buf_addr| {
                    buf_addr.decode_vec_i64(self.log_base2k, self.log_base2k*(buf_addr.cols()-1), &mut values);
                    println!("{}: {:?}", j, &values[..85]);
                    j += 1;
                });
                */

                module_lwe.vmp_prepare_dblptr(
                    &mut address.coordinates_rsh[hi].0[lo],
                    &buf_addr.dblptr(),
                    &mut buf,
                );

                buf_addr.iter_mut().for_each(|buf_addr_i| {
                    module_lwe.vec_znx_automorphism_inplace(-1, buf_addr_i, buf_addr_i.cols());
                });

                module_lwe.vmp_prepare_dblptr(
                    &mut address.coordinates_lsh[hi].0[lo],
                    &buf_addr.dblptr(),
                    &mut buf,
                );

                i += 1;
                sum_base += base;
                //println!();
            })
        });
    }

    pub fn circuit_bootstrap(
        &self,
        module: &Module,
        value: i64,
        a: &mut Vec<VecZnx>,
        tmp_bytes: &mut [u8],
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

        let cols: usize = a[0].cols();

        assert!(
            tmp_bytes.len() >= circuit_bootstrap_tmp_bytes(module, cols),
            "invalid tmp_bytes: tmp_bytes.len()={} < circuit_bootstrap_tmp_bytes={}",
            tmp_bytes.len(),
            circuit_bootstrap_tmp_bytes(module, cols)
        );

        let mut buf_pbs: VecZnx = VecZnx::from_bytes_borrow(n, cols, tmp_bytes);

        izip!(a.iter_mut(), self.test_vectors.iter()).for_each(|(ai, ti)| {
            module.vec_znx_rotate(value, &mut buf_pbs, ti);
            switch_degree(ai, &buf_pbs);
        });
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

        let step_start: usize = module.log_n() - log_gap_in as usize;
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
            let step_start: usize = 0;
            let steps: usize = min(max_value, 1 << (module.log_n() - log_gap_in as usize));

            // For each coefficients that can be packed, i.e. n / gap_in
            (0..steps).for_each(|i: usize| {
                //println!("{}", i);
                //println!("a");
                //a.print(a.cols(), a.n());

                //println!("buf0");
                //buf0.print(buf0.cols(), buf0.n());

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
