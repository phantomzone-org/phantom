use crate::address::{Address, Coordinate};
use crate::packing::StreamRepacker;
use crate::reverse_bits_msb;
use crate::trace::{trace, trace_inplace};
use base2k::{Module, VecZnx};

pub struct Memory {
    pub data: Vec<VecZnx>,
    pub log_n: usize,
    pub log_base2k: usize,
    pub log_k: usize,
}

impl Memory {
    pub fn new(log_n: usize, log_base2k: usize, log_k: usize) -> Self {
        Self {
            data: Vec::new(),
            log_n: log_n,
            log_base2k: log_base2k,
            log_k: log_k,
        }
    }

    pub fn limbs(&self) -> usize {
        (self.log_k + self.log_base2k - 1) / self.log_base2k
    }

    pub fn set(&mut self, data: &Vec<i64>) {
        let mut vectors: Vec<VecZnx> = Vec::new();
        let limbs = (self.log_k + self.log_base2k - 1) / self.log_base2k;
        for chunk in data.chunks(1 << self.log_n) {
            let mut vector: VecZnx = VecZnx::new(1 << self.log_n, self.log_base2k, limbs);
            vector.from_i64(chunk, 32, self.log_k);
            vectors.push(vector);
        }

        self.data = vectors
    }

    pub fn read(&self, module: &Module, address: &Address) -> i64 {
        let log_n: usize = module.log_n();

        let mut packer: StreamRepacker = StreamRepacker::new(module, self.log_base2k, self.limbs());
        let mut results: Vec<VecZnx> = Vec::new();

        let limbs: usize = (self.log_k + self.log_base2k - 1) / self.log_base2k;

        let mut tmp_b_dft: base2k::VecZnxDft = module.new_vec_znx_dft(limbs);
        let mut tmp_bytes: Vec<u8> =
            vec![
                u8::default();
                module.vmp_apply_dft_tmp_bytes(limbs, limbs, address.rows(), address.cols())
            ];

        let mut tmp_vec_znx: VecZnx = module.new_vec_znx(self.log_base2k, self.limbs());

        for i in 0..address.dims_n() {
            let coordinate: &Coordinate = address.at_lsh(i);

            let result_prev: &Vec<VecZnx>;

            if i == 0 {
                result_prev = &self.data;
            } else {
                result_prev = &results;
            }

            if i < address.dims_n() - 1 {
                let mut result_next: Vec<VecZnx> = Vec::new();

                // Packs the first coefficient of each polynomial.
                for chunk in result_prev.chunks(module.n()) {
                    for j in 0..module.n() {
                        let j_rev: usize = reverse_bits_msb(j, log_n as u32);
                        if j_rev < chunk.len() {
                            // Shift polynomial by X^{-idx} and then pack
                            coordinate.product(
                                &module,
                                &mut tmp_vec_znx,
                                &chunk[j_rev],
                                &mut tmp_b_dft,
                                &mut tmp_bytes,
                            );

                            packer.add(module, Some(&tmp_vec_znx), &mut result_next);
                        } else {
                            packer.add(module, None, &mut result_next);
                        }
                    }
                }

                packer.flush(module, &mut result_next);
                packer.reset();
                results = result_next.clone();
            } else {
                // Shift polynomial by X^{-idx} and then pack
                coordinate.product(
                    &module,
                    &mut tmp_vec_znx,
                    &results[0],
                    &mut tmp_b_dft,
                    &mut tmp_bytes,
                );
            }
        }
        tmp_b_dft.delete();
        tmp_vec_znx.to_i64_single(0, self.log_k)
    }

    pub fn read_and_write(
        &mut self,
        module: &Module,
        address: &Address,
        write_value: i64,
        write_bool: bool,
    ) -> i64 {
        let log_n: usize = module.log_n();

        let mut packer: StreamRepacker = StreamRepacker::new(module, self.log_base2k, self.limbs());

        let mut results: Vec<Vec<VecZnx>> = Vec::new();

        let limbs: usize = self.limbs();

        let mut tmp_a_dft: base2k::VecZnxDft = module.new_vec_znx_dft(limbs);
        let mut tmp_bytes: Vec<u8> =
            vec![
                u8::default();
                module.vmp_apply_dft_tmp_bytes(limbs, limbs, address.rows(), address.cols())
            ];

        let mut tmp_vec_znx: VecZnx = module.new_vec_znx(self.log_base2k, limbs);

        let mut buf0: VecZnx = module.new_vec_znx(self.log_base2k, limbs);
        let mut buf1: VecZnx = module.new_vec_znx(self.log_base2k, limbs);
        let mut buf2: VecZnx = module.new_vec_znx(self.log_base2k, limbs);

        //let mut coordinate_buf: Coordinate =
        //    Coordinate::new(module, address.rows(), address.cols(), address.dims_n_decomp());

        for i in 0..address.dims_n() {
            let coordinate: &Coordinate = &address.at_lsh(i);

            let result_prev: &mut Vec<VecZnx>;

            if i == 0 {
                result_prev = &mut self.data;
            } else {
                result_prev = &mut results[i - 1];
            }

            // Shift polynomial of the last iteration by X^{-i}
            result_prev.iter_mut().for_each(|poly| {
                coordinate.product_inplace(module, poly, &mut tmp_a_dft, &mut tmp_bytes);
            });

            if i < address.dims_n() - 1 {
                let mut result_next: Vec<VecZnx> = Vec::new();

                // Packs the first coefficient of each polynomial.
                for chunk in result_prev.chunks(module.n()) {
                    for i in 0..module.n() {
                        let i_rev: usize = reverse_bits_msb(i, log_n as u32);
                        if i_rev < chunk.len() {
                            packer.add(module, Some(&chunk[i_rev]), &mut result_next);
                        } else {
                            packer.add(module, None, &mut result_next)
                        }
                    }
                }

                packer.flush(module, &mut result_next);
                packer.reset();

                // Stores the packed polynomial
                results.push(result_next);
            }
        }

        let size: usize = results.len();
        let read_value: i64;

        if size != 0 {
            let result: &mut VecZnx = &mut results[size - 1][0];

            read_value = result.to_i64_single(0, self.log_k);

            // CMUX(read_value, write_value, write_bool) -> read_value/write_value
            if write_bool {
                result.from_i64_single(0, write_value, 32, self.log_k);
            }
        } else {
            read_value = self.data[0].to_i64_single(0, self.log_k);

            // CMUX(read_value, write_value, write_bool) -> read_value/write_value
            if write_bool {
                self.data[0].from_i64_single(0, write_value, 32, self.log_k)
            }
        }

        // Walk back the tree in reverse order, repacking the coefficients
        // where the read coefficient has been conditionally replaced by
        // the write value based on the write boolean.
        for i in (0..address.dims_n() - 1).rev() {
            // Index polynomial X^{-i}
            let coordinate: &Coordinate = &address.at_rsh(i + 1);

            let result_hi: &mut Vec<VecZnx>; // Above level
            let result_lo: &mut Vec<VecZnx>; // Current level

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

            // Iterates over the set of chuncks of n polynomials of the level above
            result_hi
                .chunks_mut(module.n())
                .enumerate()
                .for_each(|(j, chunk)| {
                    // Retrieve the associated polynomial to extract and pack related to the current chunk
                    let poly_lo: &mut VecZnx = &mut result_lo[j];

                    // TODO: use VmpPMat buffer to get the inverse of X^{-i}: X^{-i} -> (X^{-i})^-1 = X^{i}
                    // Apply the reverse cyclic shift to the polynomial by (X^{-i})^-1 = X^{i}
                    coordinate.product_inplace(&module, poly_lo, &mut tmp_a_dft, &mut tmp_bytes);

                    // Iterates over the polynomial of the current chunk of the level above
                    chunk.iter_mut().for_each(|poly_hi| {
                        // Extract the first coefficient poly_lo
                        // [a, b, c, d] -> [a, 0, 0, 0]
                        trace::<false>(
                            module,
                            0,
                            log_n,
                            &mut buf2,
                            poly_lo,
                            &mut tmp_vec_znx,
                            &mut tmp_bytes,
                        );

                        // Zeroes the first coefficient of poly_j
                        // [a, b, c, d] -> [0, b, c, d]
                        trace_inplace::<true>(
                            module,
                            0,
                            log_n,
                            poly_hi,
                            Some(&mut buf0),
                            &mut buf1,
                            &mut tmp_bytes,
                        );

                        // Adds TRACE(poly_lo) + TRACEINV(poly_hi)
                        module.vec_znx_add_inplace(poly_hi, &buf2);

                        // Cyclic shift poly_lo by X^-1
                        module.vec_znx_rotate_inplace(-1, poly_lo);
                    });
                });
        }

        // TODO: use VmpPMat buffer to get the inverse of X^{-i}: X^{-i} -> (X^{-i})^-1 = X^{i}
        // Apply the reverse cyclic shift to the polynomial by (X^{-i})^-1 = X^{i}
        self.data.iter_mut().for_each(|poly_lo| {
            address
                .at_rsh(0)
                .product_inplace(&module, poly_lo, &mut tmp_a_dft, &mut tmp_bytes);
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
