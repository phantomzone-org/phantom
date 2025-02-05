use base2k::{Module, VecZnx, VecZnxOps};

pub struct StreamRepacker {
    log_base2k: usize,
    accumulators: Vec<Accumulator>,
    tmp_a: VecZnx,
    tmp_b: VecZnx,
    carry: Vec<u8>,
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
    pub fn new(module: &Module, log_base2k: usize, limbs: usize) -> Self {
        let mut accumulators: Vec<Accumulator> = Vec::<Accumulator>::new();

        let log_n: usize = module.log_n();

        (0..log_n).for_each(|_| accumulators.push(Accumulator::new(module, limbs)));

        Self {
            log_base2k: log_base2k,
            accumulators: accumulators,
            tmp_a: module.new_vec_znx(limbs),
            tmp_b: module.new_vec_znx(limbs),
            carry: vec![u8::default(); module.n() * 8],
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

    pub fn add(&mut self, module: &Module, a: Option<&VecZnx>, result: &mut Vec<VecZnx>) {
        pack_core(
            module,
            self.log_base2k,
            a,
            &mut self.accumulators,
            &mut self.tmp_a,
            &mut self.tmp_b,
            &mut self.carry,
            0,
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
                self.add(module, None, result);
            }
        }
    }
}

fn pack_core(
    module: &Module,
    log_base2k: usize,
    a: Option<&VecZnx>,
    accumulators: &mut [Accumulator],
    tmp_a: &mut VecZnx,
    tmp_b: &mut VecZnx,
    carry: &mut [u8],
    i: usize,
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
        combine(
            module,
            log_base2k,
            &mut acc_prev[0],
            a,
            tmp_a,
            tmp_b,
            carry,
            i,
        );

        acc_prev[0].control = false;

        if acc_prev[0].value {
            pack_core(
                module,
                log_base2k,
                Some(&acc_prev[0].buf),
                acc_next,
                tmp_a,
                tmp_b,
                carry,
                i + 1,
            );
        } else {
            pack_core(
                module,
                log_base2k,
                None,
                acc_next,
                tmp_a,
                tmp_b,
                carry,
                i + 1,
            );
        }
    }
}

fn combine(
    module: &Module,
    log_base2k: usize,
    acc: &mut Accumulator,
    b: Option<&VecZnx>,
    tmp_a: &mut VecZnx,
    tmp_b: &mut VecZnx,
    carry: &mut [u8],
    i: usize,
) {
    let log_n = module.log_n();
    let a: &mut VecZnx = &mut acc.buf;

    let gal_el: i64;

    if i == 0 {
        gal_el = -1
    } else {
        gal_el = module.galois_element(1 << (i - 1))
    }

    if acc.value {
        a.rsh(log_base2k, 1, carry);

        if let Some(b) = b {
            // tmp_a = b * X^t
            module.vec_znx_rotate(1 << (log_n - i - 1), tmp_a, b);

            tmp_a.rsh(log_base2k, 1, carry);

            // tmp_b = a - b*X^t
            module.vec_znx_sub(tmp_b, a, tmp_a);

            // a = a + b * X^t
            module.vec_znx_add_inplace(a, tmp_a);

            // tmp_b = phi(a - b * X^t)
            module.vec_znx_automorphism_inplace(gal_el, tmp_b);

            // a = a + b * X^t + phi(a - b * X^t)
            module.vec_znx_add_inplace(a, tmp_b);
        } else {
            // tmp_a = phi(a)
            module.vec_znx_automorphism(gal_el, tmp_a, a);
            // a = a + phi(a)
            module.vec_znx_add_inplace(a, tmp_a);
        }
    } else {
        if let Some(b) = b {
            // tmp_b = b * X^t
            module.vec_znx_rotate(1 << (log_n - i - 1), tmp_b, b);
            tmp_b.rsh(log_base2k, 1, carry);

            // tmp_a = phi(b * X^t)
            module.vec_znx_automorphism(gal_el, tmp_a, tmp_b);

            // a = (b* X^t - phi(b* X^t))
            module.vec_znx_sub(a, tmp_b, tmp_a);
            acc.value = true
        }
    }
}
