use crate::test_vector::TestVector;
use math::poly::Poly;
use math::ring::Ring;

pub struct Decomposer {
    pub test_vector_quo: TestVector,
    pub test_vector_rem: Option<TestVector>,
    pub log_base: usize,
    pub buf: Poly<u64>,
}

impl Decomposer {
    pub fn new(ring: &Ring<u64>, log_base: usize) -> Self {
        let log_n: usize = ring.log_n();

        assert!(
            log_n > log_base,
            "invalid log_base: ring.log_n() <= log_base"
        );

        let log_gap: usize = log_n - log_base;
        let gap: usize = 1 << log_gap;

        let f_quo = Box::new(move |x: usize| x >> (log_n - log_base));
        let test_vector_quo: TestVector = TestVector::new(&ring, f_quo);

        let mut test_vector_rem: Option<TestVector> = None;
        if 32 % log_base != 0 {
            let f_quo = Box::new(move |x: usize| x >> (log_n - (32 % log_base)));
            test_vector_rem = Some(TestVector::new(&ring, f_quo));
        }

        Self {
            test_vector_quo,
            test_vector_rem,
            buf: ring.new_poly(),
            log_base,
        }
    }

    pub fn decompose(&mut self, ring: &Ring<u64>, value: u32) -> Vec<u64> {
        let n: usize = ring.n();

        assert!(
            n == self.test_vector_quo.n(),
            "invalid ring: ring.n()={} != self.test_vector.n()={}",
            n,
            self.test_vector_quo.n()
        );

        let log_2n: usize = ring.log_n();
        let mask_2n: u64 = 2 * n as u64 - 1;

        let d: usize = (32 + self.log_base - 1) / self.log_base;

        let mut vec: Vec<u64> = Vec::new();

        let mut value_u64: u64 = (value as u64) << 31;

        let buf: &mut Poly<u64> = &mut self.buf;

        (0..d).for_each(|i| {
            let x: i32 = ((value_u64 >> (63 - log_2n)) & mask_2n) as i32;

            if i == 0 && !self.test_vector_rem.is_none() {
                if let Some(test_vector) = &self.test_vector_rem {
                    buf.copy_from(&test_vector.0);
                }
            } else {
                buf.copy_from(&self.test_vector_quo.0);
            }

            ring.a_mul_by_x_pow_b_into_a(x, buf);

            vec.push(buf.0[0]);

            if i == 0 && !self.test_vector_rem.is_none() {
                let base_rem = 32 % self.log_base;
                value_u64 -= buf.0[0] << (63 - base_rem);
                value_u64 <<= base_rem;
            } else {
                value_u64 -= buf.0[0] << (63 - self.log_base);
                value_u64 <<= self.log_base;
            }
        });

        vec
    }
}
