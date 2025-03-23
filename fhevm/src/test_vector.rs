use base2k::{is_aligned, Infos, VecZnx};

pub struct TestVector(pub VecZnx);

impl TestVector {
    pub fn new(
        n: usize,
        f: Box<dyn Fn(i64) -> i64>,
        log_bas2k: usize,
        cols: usize,
        tmp_bytes: &mut [u8],
    ) -> Self {
        debug_assert!(is_aligned(tmp_bytes.as_ptr()));
        debug_assert!(tmp_bytes.len() == 8 * n);

        let mut test_vector: VecZnx = VecZnx::new(n, cols);

        let last: &mut [i64] = test_vector.at_mut(test_vector.cols() - 1);
        last.iter_mut().enumerate().for_each(|(i, x)| {
            *x = -f((n - i) as i64);
        });
        last[0] = f(0);

        test_vector.normalize(log_bas2k, tmp_bytes);

        Self { 0: test_vector }
    }

    pub fn n(&self) -> usize {
        self.0.n()
    }
}
