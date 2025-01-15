use crate::packing::{pack, StreamRepacker};
use math::modulus::montgomery::Montgomery;
use math::modulus::{WordOps, ONCE};
use math::poly::Poly;
use math::ring::Ring;

pub struct Memory(pub Vec<Poly<u64>>);
pub struct Index(pub Vec<Poly<u64>>);

impl Index {
    pub fn new(ring: &Ring<u64>, size: usize) -> Self {
        let log_n = ring.log_n();
        let mut polys: Vec<Poly<u64>> = Vec::new();
        let dims: usize = (size.log2() + log_n - 1) / log_n;
        (0..dims).for_each(|_| polys.push(ring.new_poly()));
        Self { 0: polys }
    }

    pub fn set(&mut self, ring: &Ring<u64>, idx: usize) {
        let log_base: usize = ring.log_n();

        //assert!(idx > (1<<log_base * self.0.len()) == 0, "invalid idx: idx={} > {}*{}={}", idx, 1<<log_base, self.0.len(), 1<<(log_base*self.0.len()));

        let mask: usize = (1 << log_base) - 1;
        let mut remain: usize = idx as _;
        let n: usize = ring.n();
        let minus_one: u64 = ring.modulus.montgomery.minus_one();

        self.0.iter_mut().for_each(|poly| {
            let chunk = remain & mask;

            poly.zero();

            if chunk != 0 {
                poly.0[n - chunk] = minus_one; // (X^i)^-1 = X^{2n-i} = -X^{n-i}
            } else {
                poly.0[0] = 1;
            }

            ring.ntt_inplace::<false>(poly);

            remain >>= log_base
        });
    }
}

impl Memory {
    pub fn new(ring: &Ring<u64>, data: &Vec<u64>) -> Self {
        let n: usize = ring.n();

        let mut polys: Vec<Poly<u64>> = Vec::new();

        for chunk in data.chunks(n) {
            let mut poly: Poly<u64> = ring.new_poly();
            poly.set(chunk);
            ring.ntt_inplace::<false>(&mut poly);
            polys.push(poly);
        }

        Self { 0: polys }
    }

    pub fn read_and_write(
        &mut self,
        ring: &Ring<u64>,
        idx: &Index,
        write_value: u64,
        write_bool: bool,
    ) -> u64 {
        let log_n: usize = ring.log_n();

        let mut packer: StreamRepacker = StreamRepacker::new(ring);

        let mut results: Vec<Vec<Poly<u64>>> = Vec::new();

        let mut buf: Poly<u64> = ring.new_poly();

        for i in 0..idx.0.len() {
            let idx_i: &Poly<u64> = &idx.0[i];

            let result_prev: &mut Vec<Poly<u64>>;

            if i == 0 {
                result_prev = &mut self.0;
            } else {
                result_prev = &mut results[i - 1];
            }

            // Shift polynomial of the last iteration by X^{-i}
            result_prev.iter_mut().for_each(|poly| {
                ring.a_mul_b_montgomery_into_a::<ONCE>(idx_i, poly);
            });

            println!("{}", i);

            if i < idx.0.len() - 1 {
                let mut result_next: Vec<Poly<u64>> = Vec::new();

                // Packs the first coefficient of each polynomial.
                for chunk in result_prev.chunks(ring.n()) {
                    for i in 0..ring.n() {
                        let i_rev: usize = i.reverse_bits_msb(log_n as u32);
                        if i_rev < chunk.len() {
                            packer.add::<true>(ring, Some(&chunk[i_rev]), &mut result_next);
                        } else {
                            packer.add::<true>(ring, None, &mut result_next)
                        }
                    }
                }

                packer.flush::<true>(ring, &mut result_next);

                result_next.iter_mut().for_each(|poly| {
                    ring.intt::<false>(poly, &mut buf);
                });

                // Stores the packed polynomial
                results.push(result_next);
            }
        }

        // READ value
        ring.intt::<false>(&results[results.len() - 1][0], &mut buf);

        let read_value = buf.0[0];

        let nth_root: usize = ring.n() << 1;
        let gal_el_inv: usize = nth_root - 1;

        for i in (0..idx.0.len()).rev() {
            let idx_i: &Poly<u64> = &idx.0[i];

            let result_prev: &mut Vec<Poly<u64>>;

            if i == 0 {
                result_prev = &mut self.0;
            } else {
                result_prev = &mut results[i - 1];
            }

            ring.a_apply_automorphism_into_b::<true>(idx_i, gal_el_inv, nth_root, &mut buf);

            // Shift polynomial of the last iteration by X^{-i}
            result_prev.iter_mut().for_each(|poly| {
                ring.a_mul_b_montgomery_into_a::<ONCE>(&buf, poly);
            });
        }

        read_value
    }
}
