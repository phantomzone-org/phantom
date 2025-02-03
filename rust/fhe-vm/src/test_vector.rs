use base2k::{Module, VecZnx};

pub struct TestVector(pub VecZnx);

impl TestVector {
    pub fn new(
        module: &Module,
        f: Box<dyn Fn(i64) -> i64>,
        log_bas2k: usize,
        limbs: usize,
    ) -> Self {
        let mut test_vector: VecZnx = module.new_vec_znx(log_bas2k, limbs);

        let last: &mut [i64] = test_vector.at_mut(test_vector.limbs() - 1);
        last.iter_mut().enumerate().for_each(|(i, x)| {
            *x = -f(-(i as i64));
        });

        let mut carry: Vec<u8> = vec![u8::default(); module.n() << 3];
        test_vector.normalize(&mut carry);

        Self { 0: test_vector }
    }

    pub fn n(&self) -> usize {
        self.0.n()
    }
}
