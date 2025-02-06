use crate::address::Address;
use crate::decompose::Decomposer;
use crate::memory::Memory;
use crate::trace::{trace, trace_inplace};
use base2k::{Encoding, Infos, Module, VecZnx, VecZnxOps, VmpPMat, VmpPMatOps};
use itertools::izip;
use std::cmp::max;

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
    ///
    /// # Example
    /// ```
    /// use base2k::{Module, VecZnx, VecZnxOps, FFT64};
    /// use fhevm::address::Address;
    /// use fhevm::circuit_bootstrapping::CircuitBootstrapper;
    /// use fhevm::memory::Memory;
    ///
    /// let n_lwe: usize = 1 << 8;
    /// let n_pbs = n_lwe << 2;
    /// let log_base2k: usize = 17;
    /// let limbs: usize = 3;
    /// let log_base_n: usize = 6;
    /// let max_address: usize = 2 * n_lwe - 37;
    /// let module_lwe: Module = Module::new::<FFT64>(n_lwe);
    ///
    /// let module_pbs: Module = Module::new::<FFT64>(n_pbs);
    ///
    /// let rows: usize = limbs;
    /// let cols: usize = limbs + 1;
    ///
    /// let acc: CircuitBootstrapper =
    ///     CircuitBootstrapper::new(&module_pbs, module_lwe.log_n(), log_base2k, cols);
    ///
    /// let value: u32 = 73;
    ///
    /// let mut buf_pbs: VecZnx = module_pbs.new_vec_znx(cols);
    ///
    /// let mut address: Address = Address::new(&module_lwe, log_base_n, max_address, rows, cols);
    ///
    /// acc.bootstrap_to_address(
    ///     &module_pbs,
    ///     &module_lwe,
    ///     value,
    ///     &mut address,
    ///     &mut buf_pbs,
    /// );
    ///
    /// let mut data: Vec<i64> = vec![i64::default(); 2 * n_lwe];
    /// data.iter_mut().enumerate().for_each(|(i, x)| *x = i as i64);
    /// let log_k = limbs * log_base2k - 5;
    /// let mut memory: Memory = Memory::new(module_lwe.log_n(), log_base2k, log_k);
    /// memory.set(&data);
    ///
    /// let out: i64 = memory.read(&module_lwe, &address);
    ///
    /// assert_eq!(out as u32, value);
    ///
    /// ```
    pub fn bootstrap_to_address(
        &self,
        module_pbs: &Module,
        module_lwe: &Module,
        value: u32,
        address: &mut Address,
        buf_pbs: &mut VecZnx,
    ) {
        // 3) LWE -> [LWE, LWE, LWE, ...]
        let mut decomposer: Decomposer = Decomposer::new(
            &module_pbs,
            &address.decomp(),
            self.log_base2k,
            address.cols,
        );

        let addr_decomp: Vec<i64> = decomposer.decompose(&module_pbs, value);

        //println!("addr_decomp: {:?}", addr_decomp);

        //println!("cols: {}", address.cols);

        // buf RGSW
        let mut buf_addr: Vec<VecZnx> = Vec::new();
        (0..address.rows).for_each(|_| buf_addr.push(module_lwe.new_vec_znx(address.cols)));

        let mut buf_post_process: [VecZnx; 4] = [
            module_lwe.new_vec_znx(address.cols),
            module_lwe.new_vec_znx(address.cols),
            module_lwe.new_vec_znx(address.cols),
            module_lwe.new_vec_znx(address.cols),
        ];

        let log_gap: usize = 3;
        let log_gap_in: usize = log_gap - (module_pbs.log_n() - module_lwe.log_n());

        let dims_n_decomp: usize = address.dims_n_decomp();

        let mut buf: Vec<u8> =
            vec![
                u8::default();
                module_lwe.vmp_prepare_contiguous_tmp_bytes(address.rows(), address.cols())
            ];

        //println!();

        let mut i: usize = 0;
        (0..address.dims_n()).for_each(|hi| {
            let mut sum_base: usize = 0;

            (0..dims_n_decomp).for_each(|lo: usize| {
                let base: usize = address.decomp_size[hi][lo];

                //println!(": {} log_gap_in: {} log_gap_out: {} value: {}", i, log_gap_in, base * (dims_n_decomp - lo-1), addr_decomp[i]);

                // 4) LWE[i] -> RGSW
                self.circuit_bootstrap(
                    module_pbs,
                    addr_decomp[i] << log_gap,
                    &mut buf_addr,
                    buf_pbs,
                );

                self.post_process(
                    module_lwe,
                    log_gap_in,
                    sum_base,
                    (1 << base) - 1,
                    &mut buf_addr,
                    &mut buf_post_process,
                );

                /*
                let mut j: usize = 0;
                buf_addr.iter_mut().for_each(|buf_addr| {
                    buf_addr.decode_vec_i64(self.log_base2k, self.log_base2k*(buf_addr.limbs()-1), &mut values);
                    println!("{}: {:?}", j, &values[..85]);
                    j += 1;
                });
                */

                module_lwe.vmp_prepare_dblptr(
                    &mut address.coordinates_rsh[hi].0[lo],
                    &buf_addr,
                    &mut buf,
                );

                buf_addr.iter_mut().for_each(|buf_addr_i| {
                    module_lwe.vec_znx_automorphism_inplace(-1, buf_addr_i);
                });

                module_lwe.vmp_prepare_dblptr(
                    &mut address.coordinates_lsh[hi].0[lo],
                    &buf_addr,
                    &mut buf,
                );

                i += 1;
                sum_base += base;
                //println!();
            })
        });
    }

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
    ///
    /// # Example
    /// ```
    /// use base2k::{Module, VecZnx, VecZnxOps, FFT64};
    /// use fhevm::address::Address;
    /// use fhevm::circuit_bootstrapping::CircuitBootstrapper;
    /// use fhevm::memory::Memory;
    ///
    /// let n_lwe: usize = 1 << 8;
    /// let n_pbs = n_lwe << 2;
    /// let log_base2k: usize = 17;
    /// let limbs: usize = 3;
    /// let log_base_n: usize = 6;
    /// let max_address: usize = 2 * n_lwe - 37;
    /// let module_lwe: Module = Module::new::<FFT64>(n_lwe);
    ///
    /// let module_pbs: Module = Module::new::<FFT64>(n_pbs);
    ///
    /// let rows: usize = limbs;
    /// let cols: usize = limbs + 1;
    ///
    /// let acc: CircuitBootstrapper =
    ///     CircuitBootstrapper::new(&module_pbs, module_lwe.log_n(), log_base2k, cols);
    ///
    /// let mut address: Address = Address::new(&module_lwe, log_base_n, max_address, rows, cols);
    ///
    /// let idx: usize = 73;
    ///
    /// address.set(&module_lwe, idx);
    ///
    /// let offset: u32 = 45;
    ///
    /// let mut buf_pbs: VecZnx = module_pbs.new_vec_znx(cols);
    ///
    /// acc.bootstrap_address(
    ///     &module_pbs,
    ///     &module_lwe,
    ///     offset,
    ///     max_address,
    ///     &mut address,
    ///     &mut buf_pbs,
    /// );
    ///
    /// let mut data: Vec<i64> = vec![i64::default(); 2 * n_lwe];
    /// data.iter_mut().enumerate().for_each(|(i, x)| *x = i as i64);
    /// let log_k = limbs * log_base2k - 5;
    /// let mut memory: Memory = Memory::new(module_lwe.log_n(), log_base2k, log_k);
    /// memory.set(&data);
    ///
    /// let out: i64 = memory.read(&module_lwe, &address);
    ///
    /// assert_eq!(out as usize, idx + offset as usize);
    ///
    /// ```
    pub fn bootstrap_address(
        &self,
        module_pbs: &Module,
        module_lwe: &Module,
        offset: u32,
        max_address: usize,
        address: &mut Address,
        buf_pbs: &mut VecZnx,
    ) {
        // 1) RGSW -> LWE
        let log_k: usize = self.log_base2k * (address.cols - 1) - 5;
        let mut mem: Memory = Memory::new(module_lwe.log_n(), self.log_base2k, log_k);
        let mut data: Vec<i64> = vec![i64::default(); max_address];
        data.iter_mut().enumerate().for_each(|(i, x)| *x = i as i64);
        mem.set(&data);

        // 2) LWE + offset
        let mut adr: i64 = mem.read(module_lwe, address);

        adr += offset as i64;

        // 3) LWE -> RGSW
        self.bootstrap_to_address(module_pbs, module_lwe, adr as u32, address, buf_pbs);
    }

    pub fn circuit_bootstrap(
        &self,
        module: &Module,
        value: i64,
        a: &mut Vec<VecZnx>,
        buf_pbs: &mut VecZnx,
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
        max_value: usize,
        a: &mut Vec<VecZnx>,
        buf: &mut [VecZnx; 4],
    ) {
        a.iter_mut().for_each(|ai| {
            self.post_process_core(module, log_gap_in, log_gap_out, max_value, ai, buf);
        });
    }

    fn post_process_core(
        &self,
        module: &Module,
        log_gap_in: usize,
        log_gap_out: usize,
        max_value: usize,
        a: &mut VecZnx,
        buf: &mut [VecZnx; 4],
    ) {
        let step_start: usize = module.log_n() - log_gap_in;
        let step_end = module.log_n();

        let [buf0, buf1, buf2, buf3] = buf;
        let mut carry: Vec<u8> = vec![u8::default(); module.n() << 3];

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
        if log_gap_in != log_gap_out {
            let step_end: usize = step_start;
            let step_start: usize = 0;
            let steps: usize = max(max_value, 1 << (module.log_n() - log_gap_in));

            // For each coefficients that can be packed, i.e. n / gap_in
            (0..steps).for_each(|i: usize| {
                // Cyclic shift the input and output by their respective X^{-gap_in} and X^{-gap_out}
                if i != 0 {
                    module.vec_znx_rotate_inplace(-(1 << log_gap_in), a);
                    module.vec_znx_rotate_inplace(-(1 << log_gap_out), buf3);
                }

                // Trace(x * X^{-gap_in}): extracts the X^{gap_in}th coefficient
                if i == 0 {
                    trace::<false>(
                        module,
                        self.log_base2k,
                        step_start,
                        step_end,
                        buf3,
                        a,
                        buf1,
                        &mut carry,
                    );
                } else {
                    trace::<false>(
                        module,
                        self.log_base2k,
                        step_start,
                        step_end,
                        buf2,
                        a,
                        buf1,
                        &mut carry,
                    );
                    module.vec_znx_add_inplace(buf3, buf2);
                }
            });

            a.copy_from(&buf3);

            a.normalize(self.log_base2k, &mut carry);

            // Cyclic shift the output back to its original position
            module.vec_znx_rotate_inplace((1 << log_gap_out) * (steps - 1) as i64, a);
        }
    }
}
