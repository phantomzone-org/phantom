use crate::address::Address;
use crate::gadget::Gadget;
use crate::packing::StreamRepacker;
use crate::trace::{a_apply_trace_into_a, a_apply_trace_into_b, gen_auto_perms};
use itertools::izip;
use math::automorphism::AutoPermMap;
use math::modulus::{WordOps, ONCE};
use math::poly::Poly;
use math::ring::Ring;

pub struct Memory {
    pub data: Vec<Poly<u64>>,
    gal_els: Vec<usize>,
    auto_perms: AutoPermMap,
}

impl Memory {
    pub fn new(ring: &Ring<u64>) -> Self {
        let (auto_perms, gal_els) = gen_auto_perms::<true>(ring);
        Self {
            data: Vec::new(),
            gal_els: gal_els,
            auto_perms: auto_perms,
        }
    }

    pub fn set(&mut self, ring: &Ring<u64>, data: &Vec<u64>) {
        let mut polys: Vec<Poly<u64>> = Vec::new();
        for chunk in data.chunks(ring.n()) {
            let mut poly: Poly<u64> = ring.new_poly();
            poly.set(chunk);
            ring.ntt_inplace::<false>(&mut poly);
            polys.push(poly);
        }

        self.data = polys
    }

    pub fn read(&self, ring: &Ring<u64>, address: &Address) -> u64 {
        let log_n: usize = ring.log_n();

        let mut packer: StreamRepacker = StreamRepacker::new(ring);
        let mut results: Vec<Poly<u64>> = Vec::new();

        let mut buf0: Poly<u64> = ring.new_poly();
        let mut buf1: Poly<u64> = ring.new_poly();
        let mut buf2: Poly<u64> = ring.new_poly();
        let mut buf3: Poly<u64> = ring.new_poly();

        for i in 0..address.dims_n() {
            let address_i: &Vec<Gadget<Poly<u64>>> = &address.gadget_matrix[i];

            let result_prev: &Vec<Poly<u64>>;

            if i == 0 {
                result_prev = &self.data;
            } else {
                result_prev = &results;
            }

            if i < address.dims_n() - 1 {
                let mut result_next: Vec<Poly<u64>> = Vec::new();

                // Packs the first coefficient of each polynomial.
                for chunk in result_prev.chunks(ring.n()) {
                    for j in 0..ring.n() {
                        let j_rev: usize = j.reverse_bits_msb(log_n as u32);
                        if j_rev < chunk.len() {
                            // Shift polynomial by X^{-i} and then pack
                            // i might be decomposed into a base smaller than N
                            address_i.iter().enumerate().for_each(|(i, gadget)| {
                                if i == 0 {
                                    gadget.product(
                                        &ring,
                                        &chunk[j_rev],
                                        &mut buf0,
                                        &mut buf1,
                                        &mut buf2,
                                        &mut buf3,
                                    );
                                } else {
                                    gadget.product_inplace(
                                        &ring, &mut buf0, &mut buf1, &mut buf2, &mut buf3,
                                    );
                                }
                            });

                            packer.add::<true>(ring, Some(&buf3), &mut result_next);
                        } else {
                            packer.add::<true>(ring, None, &mut result_next)
                        }
                    }
                }

                packer.flush::<true>(ring, &mut result_next);
                packer.reset();
                results = result_next.clone();
            } else {
                address_i.iter().enumerate().for_each(|(i, gadget)| {
                    if i == 0 {
                        gadget.product(
                            &ring,
                            &result_prev[0],
                            &mut buf0,
                            &mut buf1,
                            &mut buf2,
                            &mut buf3,
                        );
                    } else {
                        gadget.product_inplace(&ring, &mut buf0, &mut buf1, &mut buf2, &mut buf3);
                    }
                });
            }
        }

        ring.intt_inplace::<false>(&mut buf3);
        buf3.0[0]
    }

    pub fn read_and_write(
        &mut self,
        ring: &Ring<u64>,
        address: &Address,
        write_value: u64,
        write_bool: bool,
    ) -> u64 {
        let log_n: usize = ring.log_n();

        let mut packer: StreamRepacker = StreamRepacker::new(ring);

        let mut results: Vec<Vec<Poly<u64>>> = Vec::new();

        let mut buf0: Poly<u64> = ring.new_poly();
        let mut buf1: Poly<u64> = ring.new_poly();
        let mut buf2: Poly<u64> = ring.new_poly();

        let mut buf_gadget: Vec<Gadget<Poly<u64>>> = Vec::new();

        (0..address.dims_n_decomp()).for_each(|_| {
            buf_gadget.push(Gadget::new(&ring, address.log_base()));
        });

        for i in 0..address.dims_n() {
            let address_i: &Vec<Gadget<Poly<u64>>> = &address.gadget_matrix[i];

            let result_prev: &mut Vec<Poly<u64>>;

            if i == 0 {
                result_prev = &mut self.data;
            } else {
                result_prev = &mut results[i - 1];
            }

            // Shift polynomial of the last iteration by X^{-i}
            result_prev.iter_mut().for_each(|poly| {
                address_i.iter().for_each(|gadget| {
                    gadget.product_inplace(&ring, &mut buf0, &mut buf1, &mut buf2, poly);
                });
            });

            if i < address.dims_n() - 1 {
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
                packer.reset();

                // Stores the packed polynomial
                results.push(result_next);
            }
        }

        let size: usize = results.len();
        let read_value: u64;

        if size != 0 {
            let mut result = &mut results[size - 1][0];

            // READ value
            ring.intt_inplace::<false>(&mut result);

            read_value = result.0[0];

            // CMUX(read_value, write_value, write_bool) -> read_value/write_value
            if write_bool {
                result.0[0] = write_value
            }

            ring.ntt_inplace::<false>(&mut result);
        } else {
            // READ value
            ring.intt_inplace::<false>(&mut self.data[0]);

            read_value = self.data[0].0[0];

            // CMUX(read_value, write_value, write_bool) -> read_value/write_value
            if write_bool {
                self.data[0].0[0] = write_value
            }

            ring.ntt_inplace::<false>(&mut self.data[0]);
        }

        /*
        for i in 0..results.len(){
            for j in 0..results[i].len(){
                ring.intt::<false>(&results[i][j], &mut buf0);
                println!("TREE[{}][{}]: {:?}", i+1, j, buf0);
            }
            println!();
        }
         */

        let mut x_inv: Poly<u64> = ring.new_poly();
        x_inv.0[ring.n() - 1] = ring.modulus.montgomery.minus_one();
        ring.ntt_inplace::<false>(&mut x_inv);

        let gal_el_inv: usize = self.gal_els[0];

        // Walk back the tree in reverse order, repacking the coefficients
        // where the read coefficient has been conditionally replaced by
        // the write value based on the write boolean.
        for i in (0..address.dims_n() - 1).rev() {
            // Index polynomial X^{-i}
            let address_i: &Vec<Gadget<Poly<u64>>> = &address.gadget_matrix[i + 1];

            let result_hi: &mut Vec<Poly<u64>>; // Above level
            let result_lo: &mut Vec<Poly<u64>>; // Current level

            //println!("i: {}", i);

            // Top of the tree is not stored in results.
            if i == 0 {
                result_hi = &mut self.data;
                result_lo = &mut results[0];
            } else {
                let (left, right) = results.split_at_mut(i);
                result_hi = &mut left[left.len() - 1];
                result_lo = &mut right[0];
            }

            // Get the inverse of X^{-i}: X^{-i} -> (X^{-i})^-1 = X^{i}
            // Will be used to apply the reverse cyclic shift.
            if let Some(auto_perm) = self.auto_perms.get(&gal_el_inv) {
                izip!(address_i.iter(), buf_gadget.iter_mut()).for_each(|(a, b)| {
                    izip!(a.value.iter(), b.value.iter_mut()).for_each(|(a_sub, b_sub)| {
                        ring.a_apply_automorphism_from_perm_into_b::<true>(a_sub, auto_perm, b_sub);
                    });
                });
            } else {
                panic!("galois element {} not found in AutoPermMap", gal_el_inv)
            }

            // Iterates over the set of chuncks of n polynomials of the level above
            result_hi
                .chunks_mut(ring.n())
                .enumerate()
                .for_each(|(j, chunk)| {
                    // Retrieve the associated polynomial to extract and pack related to the current chunk
                    let poly_lo: &mut Poly<u64> = &mut result_lo[j];

                    // Apply the reverse cyclic shift to the polynomial by (X^{-i})^-1 = X^{i}
                    // i might be further decomposed
                    buf_gadget.iter().for_each(|gadget| {
                        gadget.product_inplace(&ring, &mut buf0, &mut buf1, &mut buf2, poly_lo);
                    });

                    // Iterates over the polynomial of the current chunk of the level above
                    chunk.iter_mut().for_each(|poly_hi| {
                        // Extract the first coefficient poly_lo
                        // [a, b, c, d] -> [a, 0, 0, 0]
                        a_apply_trace_into_b::<false, true>(
                            &ring,
                            0,
                            log_n,
                            &self.gal_els,
                            &self.auto_perms,
                            poly_lo,
                            &mut buf0,
                            &mut buf1,
                            &mut buf2,
                        );

                        // Zeroes the first coefficient of poly_j
                        // [a, b, c, d] -> [0, b, c, d]
                        a_apply_trace_into_a::<true, true>(
                            &ring,
                            0,
                            log_n,
                            &self.gal_els,
                            &self.auto_perms,
                            &mut buf0,
                            &mut buf1,
                            poly_hi,
                        );

                        // Adds TRACE(poly_lo) + TRACEINV(poly_hi)
                        ring.a_add_b_into_b::<ONCE>(&buf2, poly_hi);

                        // Cyclic shift poly_lo by X^-1
                        ring.a_mul_b_montgomery_into_a::<ONCE>(&x_inv, poly_lo);
                    });
                });
        }

        // Get the inverse of X^{-i}: X^{-i} -> (X^{-i})^-1 = X^{i}
        // Will be used to apply the reverse cyclic shift.
        if let Some(auto_perm) = self.auto_perms.get(&gal_el_inv) {
            izip!(address.gadget_matrix[0].iter(), buf_gadget.iter_mut()).for_each(|(a, b)| {
                izip!(a.value.iter(), b.value.iter_mut()).for_each(|(a_sub, b_sub)| {
                    ring.a_apply_automorphism_from_perm_into_b::<true>(a_sub, auto_perm, b_sub);
                });
            });
        }

        // Apply the reverse cyclic shift to the polynomial by (X^{-i})^-1 = X^{i}
        self.data.iter_mut().for_each(|poly_lo| {
            buf_gadget.iter().for_each(|gadget| {
                gadget.product_inplace(&ring, &mut buf0, &mut buf1, &mut buf2, poly_lo);
            });
        });

        read_value
    }
}

/*
address_i.iter().for_each(|gadget|{
                    gadget.product_inplace(
                        &ring,
                        &mut buf0,
                        &mut buf1,
                        &mut buf2,
                        poly,
                    );
                }); */
