use base2k::{
    alloc_aligned, Infos, Module, VecZnx, VecZnxApi, VecZnxBorrow, VecZnxCommon, VecZnxOps,
};

pub struct StreamRepacker {
    log_base2k: usize,
    accumulators: Vec<Accumulator>,
    tmp_bytes: Vec<u8>,
    counter: usize,
}

pub struct Accumulator {
    buf: VecZnx,
    value: bool,
    control: bool,
}

impl Accumulator {
    pub fn new(module: &Module, limbs: usize) -> Self {
        Self {
            buf: module.new_vec_znx(limbs),
            value: false,
            control: false,
        }
    }
}

impl StreamRepacker {
    pub fn new(module: &Module, log_base2k: usize, cols: usize) -> Self {
        let mut accumulators: Vec<Accumulator> = Vec::<Accumulator>::new();
        let log_n: usize = module.log_n();
        (0..log_n).for_each(|_| accumulators.push(Accumulator::new(module, cols)));
        Self {
            log_base2k: log_base2k,
            accumulators: accumulators,
            tmp_bytes: alloc_aligned(pack_core_tmp_bytes(module, cols), 64),
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

    pub fn add<T: VecZnxCommon>(
        &mut self,
        module: &Module,
        a: Option<&T>,
        result: &mut Vec<VecZnx>,
    ) {
        pack_core(
            module,
            self.log_base2k,
            a,
            &mut self.accumulators,
            0,
            &mut self.tmp_bytes,
        );
        self.counter += 1;
        if self.counter == module.n() {
            result.push(self.accumulators[module.log_n() - 1].buf.clone());
            self.reset();
        }
    }

    pub fn flush(&mut self, module: &Module, result: &mut Vec<VecZnx>) {
        if self.counter != 0 {
            while self.counter != module.n() - 1 {
                self.add(module, None::<&VecZnx>, result);
            }
        }
    }
}

pub fn pack_core_tmp_bytes(module: &Module, cols: usize) -> usize {
    2 * VecZnxBorrow::bytes_of(module.n(), cols) + module.vec_znx_normalize_base2k_tmp_bytes()
}

fn pack_core<A: VecZnxCommon>(
    module: &Module,
    log_base2k: usize,
    a: Option<&A>,
    accumulators: &mut [Accumulator],
    i: usize,
    tmp_bytes: &mut [u8],
) {
    let log_n = module.log_n();

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
        combine(module, log_base2k, &mut acc_prev[0], a, i, tmp_bytes);

        acc_prev[0].control = false;

        if acc_prev[0].value {
            pack_core(
                module,
                log_base2k,
                Some(&acc_prev[0].buf),
                acc_next,
                i + 1,
                tmp_bytes,
            );
        } else {
            pack_core(
                module,
                log_base2k,
                None::<&VecZnx>,
                acc_next,
                i + 1,
                tmp_bytes,
            );
        }
    }
}

fn combine<A: VecZnxCommon>(
    module: &Module,
    log_base2k: usize,
    acc: &mut Accumulator,
    b: Option<&A>,
    i: usize,
    tmp_bytes: &mut [u8],
) {
    let log_n = module.log_n();
    let a: &mut VecZnx = &mut acc.buf;

    let cols: usize = a.cols();
    assert!(tmp_bytes.len() >= pack_core_tmp_bytes(module, cols));

    let gal_el: i64;

    if i == 0 {
        gal_el = -1
    } else {
        gal_el = module.galois_element(1 << (i - 1))
    }

    let vec_znx_tmp_bytes = module.bytes_of_vec_znx(cols);
    let (tmp_bytes_a, tmp_bytes) = tmp_bytes.split_at_mut(vec_znx_tmp_bytes);
    let (tmp_bytes_b, tmp_bytes_carry) = tmp_bytes.split_at_mut(vec_znx_tmp_bytes);

    let mut tmp_a: VecZnxBorrow = VecZnxBorrow::from_bytes(module.n(), cols, tmp_bytes_a);
    let mut tmp_b: VecZnxBorrow = VecZnxBorrow::from_bytes(module.n(), cols, tmp_bytes_b);

    if acc.value {
        a.rsh(log_base2k, 1, tmp_bytes_carry);

        if let Some(b) = b {
            // tmp_a = b * X^t
            module.vec_znx_rotate(1 << (log_n - i - 1), &mut tmp_a, b);

            tmp_a.rsh(log_base2k, 1, tmp_bytes_carry);

            // tmp_b = a - b*X^t
            module.vec_znx_sub(&mut tmp_b, a, &tmp_a);

            // a = a + b * X^t
            module.vec_znx_add_inplace(a, &tmp_a);

            // tmp_b = phi(a - b * X^t)
            module.vec_znx_automorphism_inplace(gal_el, &mut tmp_b, cols);

            // a = a + b * X^t + phi(a - b * X^t)
            module.vec_znx_add_inplace(a, &tmp_b);
        } else {
            // tmp_a = phi(a)
            module.vec_znx_automorphism(gal_el, &mut tmp_a, a, cols);
            // a = a + phi(a)
            module.vec_znx_add_inplace(a, &tmp_a);
        }
    } else {
        if let Some(b) = b {
            // tmp_b = b * X^t
            module.vec_znx_rotate(1 << (log_n - i - 1), &mut tmp_b, b);
            tmp_b.rsh(log_base2k, 1, tmp_bytes_carry);

            // tmp_a = phi(b * X^t)
            module.vec_znx_automorphism(gal_el, &mut tmp_a, &tmp_b, cols);

            // a = (b* X^t - phi(b* X^t))
            module.vec_znx_sub(a, &tmp_b, &tmp_a);
            acc.value = true
        }
    }
}
