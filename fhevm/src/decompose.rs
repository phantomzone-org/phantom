use crate::test_vector::TestVector;
use base2k::{alloc_aligned, Encoding, Infos, Module, VecZnx, VecZnxOps};

pub struct Decomposer {
    pub buf: VecZnx,
}

pub struct Precomp {
    pub log_base2k: usize,
    pub test_vector_msb: TestVector,
    pub test_vector_quo: Vec<TestVector>,
    pub log_bases: Vec<u8>,
}

impl Precomp {
    pub fn new(n: usize, log_bases: &Vec<u8>, log_base2k: usize, cols: usize) -> Self {
        let log_n: usize = (usize::BITS - (n - 1).leading_zeros()) as _;
        let f_sign = Box::new(move |x: i64| {
            if x == 0 {
                1 << (log_n)
            } else {
                -(1 << (log_n))
            }
        });

        let mut tmp_bytes: Vec<u8> = alloc_aligned(n * std::mem::size_of::<i64>());

        let test_vector_msb: TestVector =
            TestVector::new(n, f_sign, log_base2k, cols, &mut tmp_bytes);

        let mut test_vector_quo: Vec<TestVector> = Vec::new();

        log_bases.iter().enumerate().for_each(|(i, log_base)| {
            let log_base: u8 = *log_base;
            let mut shift: u8 = 1;
            if i == log_bases.len() - 1 {
                shift = 0
            }

            let n_i64: i64 = (1 << log_n) as i64;
            let f_quo = Box::new(move |x: i64| {
                let mut y: i64 = x;
                if y < 0 {
                    let n_i64: i64 = (1 << log_n) as i64;
                    y = n_i64 + y;
                }
                (y >> (log_n as u8 - log_base - shift + 1)) << (log_n as u8 - log_base - shift + 1)
            });
            test_vector_quo.push(TestVector::new(n, f_quo, log_base2k, cols, &mut tmp_bytes))
        });

        Self {
            log_base2k,
            test_vector_msb,
            test_vector_quo,
            log_bases: log_bases.clone(),
        }
    }
}

impl Decomposer {
    pub fn new(module_pbs: &Module, cols: usize) -> Self {
        Self {
            buf: module_pbs.new_vec_znx(cols),
        }
    }

    pub fn cols(&self) -> usize {
        self.buf.cols()
    }

    pub fn decompose(&mut self, module_pbs: &Module, precomp: &Precomp, value: u32) -> Vec<i64> {
        //println!("value: {:032b}", value);

        let n: usize = module_pbs.n();

        assert!(
            n == precomp.test_vector_quo[0].0.n(),
            "invalid ring: ring.n()={} != self.test_vector.n()={}",
            n,
            precomp.test_vector_quo[0].0.n()
        );

        let log_n: usize = module_pbs.log_n();
        let log_2n: usize = log_n + 1;

        let mut vec: Vec<i64> = Vec::new();

        let mut value_u64: u64 = (value as u64) << 32;

        let mut sum_bases: u8 = 0;

        let cols = self.cols();

        //println!("log_2n: {}", log_2n);

        precomp.log_bases.iter().enumerate().for_each(|(i, base)| {
            let buf: &mut VecZnx = &mut self.buf;

            //assert!(
            //    log_2n - 2 > *base,
            //    "invalid module_pbs: log_2n={} < base+2={}",
            //    log_2n,
            //    base + 2
            //);

            let last: bool = i == precomp.log_bases.len() - 1;

            sum_bases += *base;

            //println!("{} {}", sum_bases, base);

            //println!(
            //    "before         : {:032b} {:032b}",
            //    value_u64 >> 32,
            //    value_u64 & 0xffffffff
            //);

            // 1) From mod Q to mod 2N, with scaling by drift = N/Base
            // Example:
            //                     IGNORED            MSB  BASE     GAP   ERROR
            // x mod Q : [11110000111100001111000011] [1] [11111] [0...0] [e..e]
            //
            //           MSB  BASE     DRIFT
            // x mod 2N: [1] [111111] [00000]
            let mut shift: u8 = 32 - sum_bases;

            if !last {
                shift -= 1
            }

            let mut x: i32 = ((value_u64 << shift) >> (64 - log_2n)) as i32;

            //println!("x              : {:032b} {:032b}", 0, x);

            // 2) Padd with drift/2 such that value cannot be negative
            // [1] [111111] [00000] -> [1] [111111] [10000]
            x += 1 << (log_2n as u8 - base - 2);

            //println!("extrac & pad   : {:032b} {:032b}", 0, x);

            // 3) PBS to extract msb
            // [1] [111111] [10000] -> [1] [00000] [00000]
            module_pbs.vec_znx_rotate(x as i64, buf, &precomp.test_vector_msb.0);

            // 4) Subtracts msb from x
            // [1] [111111] [10000] ->  [0] [111111] [10000]
            let sign_bit: u64 =
                ((buf.decode_coeff_i64(precomp.log_base2k, cols * precomp.log_base2k, 0)
                    + (1 << log_n))
                    >> 1) as u64;

            //println!("    sign(x)    : {:032b} {:032b}", 0, sign_bit);

            x -= sign_bit as i32;

            //println!("x - sign(x)    : {:032b} {:032b}", 0, x);

            // 5) PBS bit-extraction
            // [0] [111111] [10000] ->  [0] [111111] [00000]
            module_pbs.vec_znx_rotate(x as i64, buf, &precomp.test_vector_quo[i].0);

            // Adds back MSB if this is the last iteration
            let mut digits: u64 =
                buf.decode_coeff_i64(precomp.log_base2k, cols * precomp.log_base2k, 0) as u64;

            if last {
                digits += sign_bit;
            }

            //println!(
            //    "digits         : {:032b} {:032b}",
            //    digits >> 32,
            //    digits & 0xffffffff
            //);

            // Stores i-th diit
            if last {
                vec.push((digits >> (log_2n as u8 - base)) as i64);
            } else {
                vec.push((digits >> (log_2n as u8 - base - 1)) as i64);
            }

            //println!("out            : {:032b} {:032b}", vec[i]>>32, vec[i]&0xffffffff);
            //println!("value_u64      : {:032b} {:032b}", value_u64>>32, value_u64&0xffffffff);

            // 6) Subtracts i-th digit to prepare for next iteration
            // x mod Q : [11110000111100001111000011] [1] [11111] [0...0] [e..e]
            //         - [00000000000000000000000000] [0] [11111] [0...0] [e..e]
            //         =
            // x mod Q : [11110000111100001111000011] [1] [00000] [0...0] [e..e]

            if last {
                digits = digits << (32 - log_2n as u8 + sum_bases);
            } else {
                digits = digits << (32 - log_2n as u8 + sum_bases + 1);
            }

            //println!("digit final    : {:032b} {:032b}", digits>>32, digits&0xffffffff);

            value_u64 -= digits;

            //println!("value_u64 final: {:032b} {:032b}", value_u64>>32, value_u64&0xffffffff);
            //println!();
        });

        vec
    }
}
