use crate::address::{Address, Coordinate};
use crate::packing::StreamRepacker;
use crate::reverse_bits_msb;
use crate::trace::{trace, trace_inplace_inv, trace_inv_tmp_bytes};
use base2k::{Encoding, Infos, Module, VecZnx, VecZnxDft, VecZnxDftOps, VecZnxOps, VmpPMatOps};
use itertools::izip;

pub struct Memory {
    pub data: Vec<VecZnx>,
    pub n: usize,
    pub log_base2k: usize,
    pub log_k: usize,
    pub cols: usize,
    pub max_size: usize,
    pub tree: Vec<Vec<VecZnx>>,
    pub state: bool,
}

pub fn read_tmp_bytes(
    module: &Module,
    cols: usize,
    address_rows: usize,
    address_cols: usize,
) -> usize {
    let mut tmp_bytes: usize = 0;
    tmp_bytes += module.bytes_of_vec_znx(cols);
    tmp_bytes += module.bytes_of_vec_znx_dft(cols);
    tmp_bytes += module.vmp_apply_dft_tmp_bytes(cols, cols, address_rows, address_cols);
    tmp_bytes
}

pub fn read_prepare_write_tmp_bytes(
    module: &Module,
    cols: usize,
    address_rows: usize,
    address_cols: usize,
) -> usize {
    let mut tmp_bytes: usize = 0;
    tmp_bytes += module.bytes_of_vec_znx_dft(cols);
    tmp_bytes += module.vmp_apply_dft_tmp_bytes(cols, cols, address_rows, address_cols);
    tmp_bytes
}

pub fn write_tmp_bytes(
    module: &Module,
    cols: usize,
    address_rows: usize,
    address_cols: usize,
) -> usize {
    let mut tmp_bytes: usize = 0;
    tmp_bytes += 2 * module.bytes_of_vec_znx(cols);
    tmp_bytes += trace_inv_tmp_bytes(module, cols)
        | module.vmp_apply_dft_tmp_bytes(cols, cols, address_rows, address_cols);
    tmp_bytes += module.bytes_of_vec_znx_dft(cols);
    tmp_bytes
}

impl Memory {
    pub fn new(module: &Module, log_base2k: usize, cols: usize, max_size: usize) -> Self {
        let n: usize = module.n();
        let mut tree: Vec<Vec<VecZnx>> = Vec::new();

        if max_size > n {
            let mut size: usize = (max_size + n - 1) / n; // Skip first recursion as it is stored in data

            while size != 1 {
                size = (size + n - 1) / n;
                let mut tmp: Vec<VecZnx> = Vec::new();
                (0..size).for_each(|_| {
                    tmp.push(module.new_vec_znx(cols));
                });
                tree.push(tmp);
            }
        }

        Self {
            data: Vec::new(),
            n: n,
            log_base2k: log_base2k,
            cols: cols,
            log_k: 0,
            max_size: max_size,
            tree: tree,
            state: false,
        }
    }

    pub fn print(&self) {
        let n: usize = self.data[0].n();
        let mut values: Vec<i64> = vec![0i64; n];
        'outer: for i in 0..self.data.len() {
            self.data[i].decode_vec_i64(self.log_base2k, self.log_k, &mut values);
            for j in 0..n {
                print!("{:04}: {}\n", i * n + j, values[j]);
                if i * n + j > self.max_size {
                    break 'outer;
                }
            }
        }
    }

    pub fn set(&mut self, data: &[i64], log_k: usize) {
        assert!(
            data.len() <= self.max_size,
            "invalid data: data.len()={} > self.max_size={}",
            data.len(),
            self.max_size
        );
        let mut vectors: Vec<VecZnx> = Vec::new();
        for chunk in data.chunks(self.n) {
            let mut vector: VecZnx = VecZnx::new(self.n, self.cols);
            vector.encode_vec_i64(self.log_base2k, log_k, chunk, 32);
            vectors.push(vector);
        }
        self.data = vectors;
        self.log_k = log_k;
    }

    pub fn read(&self, module: &Module, address: &Address, tmp_bytes: &mut [u8]) -> u32 {
        assert_eq!(
            self.state, false,
            "invalid call to Memory.read: internal state is true -> requires calling Memory.write"
        );
        assert!(
            tmp_bytes.len() >= read_tmp_bytes(module, self.cols, address.rows(), address.cols()),
            "invalid tmp_bytes: must be of size greater or equal to self.read_tmp_bytes"
        );

        let log_n: usize = module.log_n();

        let mut packer: StreamRepacker = StreamRepacker::new(module, self.log_base2k, self.cols);
        let mut results: Vec<VecZnx> = Vec::new();

        let cols: usize = self.cols;

        let bytes_of_vec_znx: usize = module.bytes_of_vec_znx(cols);
        let bytes_of_vec_znx_dft: usize = module.bytes_of_vec_znx_dft(cols);
        let (tmp_bytes_vec_znx, tmp_bytes) = tmp_bytes.split_at_mut(bytes_of_vec_znx);
        let (tmp_bytes_vec_znx_dft, tmp_bytes_apply_dft) =
            tmp_bytes.split_at_mut(bytes_of_vec_znx_dft);

        let mut tmp_vec_znx: VecZnx =
            VecZnx::from_bytes_borrow(1 << log_n, cols, tmp_bytes_vec_znx);
        let mut tmp_b_dft: base2k::VecZnxDft =
            VecZnxDft::from_bytes_borrow(module, cols, tmp_bytes_vec_znx_dft);

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
                                self.log_base2k,
                                &mut tmp_vec_znx,
                                &chunk[j_rev],
                                &mut tmp_b_dft,
                                tmp_bytes_apply_dft,
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
                if i == 0 {
                    // Shift polynomial by X^{-idx} and then pack
                    coordinate.product(
                        &module,
                        self.log_base2k,
                        &mut tmp_vec_znx,
                        &self.data[0],
                        &mut tmp_b_dft,
                        tmp_bytes_apply_dft,
                    );
                } else {
                    // Shift polynomial by X^{-idx} and then pack
                    coordinate.product(
                        &module,
                        self.log_base2k,
                        &mut tmp_vec_znx,
                        &results[0],
                        &mut tmp_b_dft,
                        tmp_bytes_apply_dft,
                    );
                }
            }
        }
        tmp_vec_znx.decode_coeff_i64(self.log_base2k, self.log_k, 0) as u32
    }

    pub fn read_prepare_write(
        &mut self,
        module: &Module,
        address: &Address,
        tmp_bytes: &mut [u8],
    ) -> u32 {
        assert_eq!(self.state, false, "invalid call to Memory.read: internal state is true -> requires calling Memory.write_after_read");
        assert!(tmp_bytes.len() >= read_prepare_write_tmp_bytes(module, self.cols, address.rows(), address.cols()), "invalid tmp_bytes: must be of size greater or equal to self.read_prepare_write_tmp_bytes");

        let log_n: usize = module.log_n();

        let mut packer: StreamRepacker = StreamRepacker::new(module, self.log_base2k, self.cols);

        let cols: usize = self.cols;

        let bytes_of_vec_znx_dft: usize = module.bytes_of_vec_znx_dft(cols);
        let (tmp_bytes_vec_znx_dft, tmp_bytes_apply_dft) =
            tmp_bytes.split_at_mut(bytes_of_vec_znx_dft);
        let mut tmp_a_dft: base2k::VecZnxDft =
            VecZnxDft::from_bytes_borrow(module, cols, tmp_bytes_vec_znx_dft);

        //let mut coordinate_buf: Coordinate =
        //    Coordinate::new(module, address.rows(), address.cols(), address.dims_n_decomp());

        for i in 0..address.dims_n() {
            let coordinate: &Coordinate = &address.at_lsh(i);

            let result_prev: &mut Vec<VecZnx>;

            if i == 0 {
                result_prev = &mut self.data;
            } else {
                result_prev = &mut self.tree[i - 1];
            }

            // Shift polynomial of the last iteration by X^{-i}
            result_prev.iter_mut().for_each(|poly| {
                coordinate.product_inplace(
                    module,
                    self.log_base2k,
                    poly,
                    &mut tmp_a_dft,
                    tmp_bytes_apply_dft,
                );
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
                            packer.add(module, None::<&VecZnx>, &mut result_next)
                        }
                    }
                }

                packer.flush(module, &mut result_next);
                packer.reset();

                // Stores the packed polynomial
                izip!(self.tree[i].iter_mut(), result_next.iter()).for_each(|(a, b)| {
                    a.copy_from(b);
                });
            }
        }

        self.state = true;

        if address.dims_n() != 1 {
            return self.tree.last_mut().unwrap()[0].decode_coeff_i64(self.log_base2k, self.log_k, 0)
                as u32;
        }

        self.data[0].decode_coeff_i64(self.log_base2k, self.log_k, 0) as u32
    }

    pub fn write(
        &mut self,
        module: &Module,
        address: &Address,
        write_value: u32,
        tmp_bytes: &mut [u8],
    ) {
        assert_eq!(self.state, true, "invalid call to Memory.read: internal state is true -> requires calling Memory.write_after_read");
        assert!(
            tmp_bytes.len() >= write_tmp_bytes(module, self.cols, address.rows(), address.cols()),
            "invalid tmp_bytes: must be of size greater or equal to self.write_tmp_bytes"
        );

        let log_n: usize = module.log_n();

        let cols: usize = self.cols;

        let bytes_of_vec_znx: usize = module.bytes_of_vec_znx(cols);
        let bytes_of_vec_znx_dft: usize = module.bytes_of_vec_znx_dft(cols);

        let (tmp_bytes_vec_znx, tmp_bytes) = tmp_bytes.split_at_mut(bytes_of_vec_znx);
        let (tmp_bytes_vec_znx_dft, tmp_bytes) = tmp_bytes.split_at_mut(bytes_of_vec_znx_dft);

        let mut tmp_a: VecZnx = VecZnx::from_bytes_borrow(1 << log_n, cols, tmp_bytes_vec_znx);
        let mut tmp_a_dft: base2k::VecZnxDft =
            VecZnxDft::from_bytes_borrow(module, cols, tmp_bytes_vec_znx_dft);

        if address.dims_n() != 1 {
            let result: &mut VecZnx = &mut self.tree.last_mut().unwrap()[0];
            result.encode_coeff_i64(self.log_base2k, self.log_k, 0, write_value as i64, 32);
            result.normalize(self.log_base2k, tmp_bytes);
        } else {
            self.data[0].encode_coeff_i64(self.log_base2k, self.log_k, 0, write_value as i64, 32);
            self.data[0].normalize(self.log_base2k, tmp_bytes);
        }

        // Walk back the tree in reverse order, repacking the coefficients
        // where the read coefficient has been conditionally replaced by
        // the write value based on the write boolean.
        for i in (0..address.dims_n() - 1).rev() {
            // Index polynomial X^{-i}
            let coordinate: &Coordinate = &address.at_rsh(i + 1);

            let result_hi: &mut Vec<VecZnx>; // Above level
            let result_lo: &mut Vec<VecZnx>; // Current level

            // Top of the tree is not stored in results.
            if i == 0 {
                result_hi = &mut self.data;
                result_lo = &mut self.tree[0];
            } else {
                let (left, right) = self.tree.split_at_mut(i);
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
                    coordinate.product_inplace(
                        &module,
                        self.log_base2k,
                        poly_lo,
                        &mut tmp_a_dft,
                        tmp_bytes,
                    );

                    // Iterates over the polynomial of the current chunk of the level above
                    chunk.iter_mut().for_each(|poly_hi| {
                        // Extract the first coefficient poly_lo
                        // [a, b, c, d] -> [a, 0, 0, 0]
                        trace(
                            module,
                            self.log_base2k,
                            0,
                            log_n,
                            &mut tmp_a,
                            poly_lo,
                            tmp_bytes,
                        );
                        tmp_a.normalize(self.log_base2k, tmp_bytes);

                        // Zeroes the first coefficient of poly_j
                        // [a, b, c, d] -> [0, b, c, d]
                        trace_inplace_inv(module, self.log_base2k, 0, log_n, poly_hi, tmp_bytes);

                        // Adds TRACE(poly_lo) + TRACEINV(poly_hi)
                        module.vec_znx_add_inplace(poly_hi, &tmp_a);

                        // Cyclic shift poly_lo by X^-1
                        module.vec_znx_rotate_inplace(-1, poly_lo);
                    });
                });
        }

        // TODO: use VmpPMat buffer to get the inverse of X^{-i}: X^{-i} -> (X^{-i})^-1 = X^{i}
        // Apply the reverse cyclic shift to the polynomial by (X^{-i})^-1 = X^{i}
        self.data.iter_mut().for_each(|poly_lo| {
            address.at_rsh(0).product_inplace(
                &module,
                self.log_base2k,
                poly_lo,
                &mut tmp_a_dft,
                tmp_bytes,
            );
        });

        self.state = false;
    }
}
