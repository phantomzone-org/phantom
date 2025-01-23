use crate::test_vector::TestVector;
use math::poly::Poly;
use math::ring::Ring;

pub struct Decomposer {
    pub test_vector_msb: TestVector,
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

        let f_sign = Box::new(move |x: usize| (x >> (log_n - 1)) << (log_n - 1));
        let test_vector_msb: TestVector = TestVector::new(&ring, f_sign);

        let f_quo =
            Box::new(move |x: usize| (x >> (log_n - log_base - 1)) << (log_n - log_base - 1));
        let test_vector_quo: TestVector = TestVector::new(&ring, f_quo);

        let mut test_vector_rem: Option<TestVector> = None;
        if 32 % log_base != 0 {
            let f_quo = Box::new(move |x: usize| {
                x >> (log_n - (32 % log_base) - 1) << (log_n - (32 % log_base) - 1)
            });
            test_vector_rem = Some(TestVector::new(&ring, f_quo));
        }

        Self {
            test_vector_msb,
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

        let d: usize = (32 + self.log_base - 1) / self.log_base;

        let mut vec: Vec<u64> = Vec::new();

        let mut value_u64: u64 = (value as u64) << 32;

        let buf: &mut Poly<u64> = &mut self.buf;

        (0..d).for_each(|i| {
            //println!("before         : {:032b} {:032b}", value_u64>>32, value_u64&0xffffffff);

            // 1) From mod Q to mod 2N, with scaling by drift = N/Base
            // Example:
            //                     IGNORED            MSB  BASE     GAP   ERROR
            // x mod Q : [11110000111100001111000011] [1] [11111] [0...0] [e..e]
            //
            //           MSB  BASE     DRIFT
            // x mod 2N: [1] [111111] [00000]
            let mut x: i32;
            if i == d - 1 && !self.test_vector_rem.is_none() {
                x = (value_u64 >> (64 - log_2n)) as i32;
            } else {
                x = ((value_u64 << (32 - (i + 1) * self.log_base) - 1) >> (64 - log_2n)) as i32;
            }

            // 2) Padd with drift/2 such that value cannot be negative
            // [1] [111111] [00000] -> [1] [111111] [10000]
            x += 1 << (log_2n - self.log_base - 2);

            //println!("extrac & pad   : {:032b} {:032b}", 0, x);

            // 3) PBS to extract msb
            // [1] [111111] [10000] -> [1] [00000] [00000]
            buf.copy_from(&self.test_vector_msb.0);
            ring.a_mul_by_x_pow_b_into_a(x, buf);

            // 4) Subtracts msb from x
            // [1] [111111] [10000] ->  [0] [111111] [10000]
            let sign_bit: u64 = buf.0[0];
            x -= sign_bit as i32;

            //println!("x - sign(x)    : {:032b} {:032b}", 0, x);

            // 5) PBS bit-extraction
            // [0] [111111] [10000] ->  [0] [111111] [00000]
            if i == d - 1 && !self.test_vector_rem.is_none() {
                if let Some(test_vector) = &self.test_vector_rem {
                    buf.copy_from(&test_vector.0);
                }
            } else {
                buf.copy_from(&self.test_vector_quo.0);
            }
            ring.a_mul_by_x_pow_b_into_a(x, buf);

            // Adds back MSB if this is the last iteration
            let mut digits: u64 = buf.0[0];
            if i == d - 1 {
                digits += sign_bit;
            }

            // Stores i-th diit
            if i == d - 1 && !self.test_vector_rem.is_none() {
                vec.push(digits >> (log_2n - (32 % self.log_base)));
            } else {
                vec.push(digits >> (log_2n - self.log_base - 1));
            }

            //println!("out            : {:032b} {:032b}", vec[i]>>32, vec[i]&0xffffffff);
            //println!("digits         : {:032b} {:032b}", digits>>32, digits&0xffffffff);
            //println!("value_u64      : {:032b} {:032b}", value_u64>>32, value_u64&0xffffffff);

            // 6) Subtracts i-th digit to prepare for next iteration
            // x mod Q : [11110000111100001111000011] [1] [11111] [0...0] [e..e]
            //         - [00000000000000000000000000] [0] [11111] [0...0] [e..e]
            //         =
            // x mod Q : [11110000111100001111000011] [1] [00000] [0...0] [e..e]
            let k: u64;
            if i < d - 1 && !self.test_vector_rem.is_none() {
                k = digits << 32 - log_2n + (i + 1) * self.log_base + 1;
            } else {
                k = digits << (64 - log_2n);
            }

            value_u64 -= k;

            //println!("k              : {:032b} {:032b}", k>>32, k&0xffffffff);
            //println!("value_u64 final: {:032b} {:032b}", value_u64>>32, value_u64&0xffffffff);
            //println!();
        });

        vec
    }
}
