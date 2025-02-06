use base2k::{Infos, Module, VecZnx, VecZnxOps};

pub struct TestVector(pub VecZnx);

impl TestVector {
    pub fn new(
        module: &Module,
        f: Box<dyn Fn(i64) -> i64>,
        log_bas2k: usize,
        limbs: usize,
    ) -> Self {
        let n: i64 = module.n() as i64;
        let mut test_vector: VecZnx = module.new_vec_znx(limbs);

        let last: &mut [i64] = test_vector.at_mut(test_vector.limbs() - 1);
        last.iter_mut().enumerate().for_each(|(i, x)| {
            *x = -f(n - i as i64);
        });
        last[0] = f(0);

        let mut carry: Vec<u8> = vec![u8::default(); module.n() << 3];
        test_vector.normalize(log_bas2k, &mut carry);

        Self { 0: test_vector }
    }

    pub fn n(&self) -> usize {
        self.0.n()
    }
}
