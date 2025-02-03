use crate::test_vector::{self, TestVector};
use rns::poly::Poly;
use rns::ring::Ring;

pub struct Decomposer {
    pub test_vector_msb: TestVector,
    pub test_vector_quo: Vec<TestVector>,
    pub log_bases: Vec<usize>,
    pub buf: Poly<u64>,
}

/*
impl Decomposer {
    pub fn new(ring: &Ring<u64>, log_bases: &Vec<usize>) -> Self {
        let log_n: usize = ring.log_n();

        let f_sign = Box::new(move |x: usize| (x >> (log_n - 1)) << (log_n - 1));
        let test_vector_msb: TestVector = TestVector::new(&ring, f_sign);

        let mut test_vector_quo: Vec<TestVector> = Vec::new();

        log_bases.iter().enumerate().for_each(|(i, log_base)| {
            let log_base = *log_base;
            let mut shift: usize = 1;
            if i == log_bases.len() - 1 {
                shift = 0
            }
            let f_quo = Box::new(move |x: usize| {
                (x >> (log_n - log_base - shift)) << (log_n - log_base - shift)
            });
            test_vector_quo.push(TestVector::new(&ring, f_quo))
        });

        Self {
            test_vector_msb,
            test_vector_quo,
            buf: ring.new_poly(),
            log_bases: log_bases.clone(),
        }
    }

    pub fn decompose(&mut self, ring: &Ring<u64>, value: u32) -> Vec<u64> {
        let n: usize = ring.n();

        assert!(
            n == self.test_vector_quo[0].0.n(),
            "invalid ring: ring.n()={} != self.test_vector.n()={}",
            n,
            self.test_vector_quo[0].0.n()
        );

        let log_2n: usize = ring.log_n();

        let mut vec: Vec<u64> = Vec::new();

        let mut value_u64: u64 = (value as u64) << 32;

        let buf: &mut Poly<u64> = &mut self.buf;

        let mut sum_bases: usize = 0;

        self.log_bases.iter().enumerate().for_each(|(i, base)| {
            let last: bool = i == self.log_bases.len() - 1;

            sum_bases += *base;

            println!("{} {}", sum_bases, base);

            println!(
                "before         : {:032b} {:032b}",
                value_u64 >> 32,
                value_u64 & 0xffffffff
            );

            // 1) From mod Q to mod 2N, with scaling by drift = N/Base
            // Example:
            //                     IGNORED            MSB  BASE     GAP   ERROR
            // x mod Q : [11110000111100001111000011] [1] [11111] [0...0] [e..e]
            //
            //           MSB  BASE     DRIFT
            // x mod 2N: [1] [111111] [00000]
            let mut shift: usize = 32 - sum_bases;

            if !last {
                shift -= 1
            }

            println!("shift {}", shift);

            let mut x: i32 = ((value_u64 << shift) >> (64 - log_2n)) as i32;

            println!("x              : {:032b} {:032b}", 0, x);

            // 2) Padd with drift/2 such that value cannot be negative
            // [1] [111111] [00000] -> [1] [111111] [10000]
            x += 1 << (log_2n - base - 2);

            println!("extrac & pad   : {:032b} {:032b}", 0, x);

            // 3) PBS to extract msb
            // [1] [111111] [10000] -> [1] [00000] [00000]
            buf.copy_from(&self.test_vector_msb.0);
            ring.a_mul_by_x_pow_b_into_a(x, buf);

            // 4) Subtracts msb from x
            // [1] [111111] [10000] ->  [0] [111111] [10000]
            let sign_bit: u64 = buf.0[0];
            x -= sign_bit as i32;

            println!("x - sign(x)    : {:032b} {:032b}", 0, x);

            // 5) PBS bit-extraction
            // [0] [111111] [10000] ->  [0] [111111] [00000]
            buf.copy_from(&self.test_vector_quo[i].0);
            ring.a_mul_by_x_pow_b_into_a(x, buf);

            // Adds back MSB if this is the last iteration
            let mut digits: u64 = buf.0[0];
            if last {
                digits += sign_bit;
            }

            println!(
                "digits         : {:032b} {:032b}",
                digits >> 32,
                digits & 0xffffffff
            );

            // Stores i-th diit
            if last {
                vec.push(digits >> (log_2n - base));
            } else {
                vec.push(digits >> (log_2n - base - 1));
            }

            //println!("out            : {:032b} {:032b}", vec[i]>>32, vec[i]&0xffffffff);

            //println!("value_u64      : {:032b} {:032b}", value_u64>>32, value_u64&0xffffffff);

            // 6) Subtracts i-th digit to prepare for next iteration
            // x mod Q : [11110000111100001111000011] [1] [11111] [0...0] [e..e]
            //         - [00000000000000000000000000] [0] [11111] [0...0] [e..e]
            //         =
            // x mod Q : [11110000111100001111000011] [1] [00000] [0...0] [e..e]
            value_u64 -= digits << (32 - log_2n + sum_bases + 1);

            //println!("k              : {:032b} {:032b}", k>>32, k&0xffffffff);
            //println!("value_u64 final: {:032b} {:032b}", value_u64>>32, value_u64&0xffffffff);
            //println!();
        });

        vec
    }
}
*/
