use base2k::{
    alloc_aligned, alloc_aligned_u8, Infos, Module, VecZnx, VecZnxBig, VecZnxBigOps, VecZnxDft,
    VecZnxDftOps, VmpPMat, VmpPMatOps,
};
use itertools::izip;

use crate::{
    decompose::Decomp,
    memory::{read_tmp_bytes, Memory},
};

pub struct Address {
    pub rows: usize,
    pub cols: usize,
    pub coordinates_lsh: Vec<Coordinate>,
    pub coordinates_rsh: Vec<Coordinate>,
    pub decomp: Decomp,
}

impl Address {
    pub fn new(module: &Module, decomp: &Decomp, rows: usize, cols: usize) -> Self {
        assert!(
            module.n() >=
            decomp.max_n1(),
            "inner_decomp={} is smaller than ring_degree={}",
            decomp.max_n1(),
            module.n()
        );

        let mut coordinates_lsh: Vec<Coordinate> = Vec::new();
        let mut coordinates_rsh: Vec<Coordinate> = Vec::new();
        (0..decomp.n1()).for_each(|i| {
            coordinates_lsh.push(Coordinate::new(module, rows, cols, decomp));
            coordinates_rsh.push(Coordinate::new(module, rows, cols, decomp));
        });
        Self {
            rows: rows,
            cols: cols,
            coordinates_lsh: coordinates_lsh,
            coordinates_rsh: coordinates_rsh,
            decomp: decomp.clone(),
        }
    }

    pub fn rows(&self) -> usize {
        self.coordinates_rsh[0].value[0].rows()
    }

    pub fn cols(&self) -> usize {
        self.coordinates_rsh[0].value[0].cols()
    }

    pub fn n1(&self) -> usize {
        self.coordinates_rsh.len()
    }

    pub fn n2(&self) -> usize {
        self.coordinates_rsh[0].value.len()
    }

    pub fn set(&mut self, module: &Module, idx: u32) {
        debug_assert!(self.decomp.max() > idx as usize);
        let max_n1: usize = self.decomp.max_n1();
        let mask_n1: usize = max_n1 - 1;
        let mut remain: usize = idx as _;

        izip!(
            self.coordinates_lsh.iter_mut(),
            self.coordinates_rsh.iter_mut(),
        )
        .for_each(|(coordinate_lsh, coordinate_rsh)| {
            let k: usize = remain & mask_n1;
            coordinate_lsh.encode(module, -(k as i64));
            coordinate_rsh.encode(module, k as i64);
            remain /= max_n1;
        })
    }

    pub fn at_lsh(&self, i: usize) -> &Coordinate {
        &self.coordinates_lsh[i]
    }

    pub fn at_rsh(&self, i: usize) -> &Coordinate {
        &self.coordinates_rsh[i]
    }

    pub fn max(&self) -> usize {
        self.decomp.max()
    }

    pub fn debug_as_u32(&self, module: &Module) -> u32 {
        let cols: usize = 3;
        let mut mem: Memory = Memory::new(module, 16, cols, self.max());
        let mut data: Vec<i64> = vec![0; self.max()];
        data.iter_mut().enumerate().for_each(|(i, x)| *x = i as i64);
        mem.set(&data, 32);
        let mut tmp_bytes: Vec<u8> =
            alloc_aligned(read_tmp_bytes(module, cols, self.rows(), self.cols()));
        mem.read(module, &self, &mut tmp_bytes)
    }
}

pub struct Coordinate {
    pub value: Vec<VmpPMat>,
    pub decomp: Vec<u8>,
    pub gap: usize,
}

impl Coordinate {
    pub fn new(module: &Module, rows: usize, cols: usize, decomp: &Decomp) -> Self {
        let mut coordinates: Vec<VmpPMat> = Vec::new();
        (0..decomp.n2()).for_each(|_| coordinates.push(module.new_vmp_pmat(rows, cols)));
        Self {
            value: coordinates,
            decomp: decomp.base.clone(),
            gap: decomp.gap(module.log_n()),
        }
    }

    pub fn n2(&self) -> usize {
        self.value.len()
    }

    pub fn encode(&mut self, module: &Module, value: i64) {
        let n: usize = module.n();
        let rows: usize = self.value[0].rows();
        let cols: usize = self.value[0].cols();

        let sign: i64 = value.signum();
        let mut remain: usize = value.abs() as usize;

        let mut tmp_bytes: Vec<u8> = alloc_aligned_u8(module.vmp_prepare_tmp_bytes(rows, cols));
        let mut buf_i64: Vec<i64> = alloc_aligned::<i64>(n * cols);

        let mut tot_base: u8 = 0;
        izip!(self.value.iter_mut(), self.decomp.iter()).for_each(|(vmp_pmat, base)| {
            let mask: usize = (1 << base) - 1;

            let chunk: usize = ((remain & mask) << tot_base) * self.gap;

            //println!(
            //    "value: {}, remain: {} base: {} tot_base: {} chunk: {} mask: {} gap: {}",
            //    value, remain, base, tot_base, chunk, mask, self.gap
            //);

            (0..rows).for_each(|row_i| {
                let offset: usize = n * row_i;

                if sign < 0 && chunk != 0 {
                    buf_i64[offset + n - chunk] = -1; // (X^i)^-1 = X^{2n-i} = -X^{n-i}
                } else {
                    buf_i64[offset + chunk] = 1;
                }

                module.vmp_prepare_row(vmp_pmat, &buf_i64, row_i, &mut tmp_bytes);

                if sign < 0 && chunk != 0 {
                    buf_i64[offset + n - chunk] = 0;
                } else {
                    buf_i64[offset + chunk] = 0;
                }
            });

            remain >>= base;
            tot_base += base
        });
    }

    pub fn product(
        &self,
        module: &Module,
        log_base2k: usize,
        b: &mut VecZnx,
        a: &VecZnx,
        tmp_b_dft: &mut VecZnxDft,
        buf: &mut [u8],
    ) {
        self.value.iter().enumerate().for_each(|(i, vmp_pmat)| {
            if i == 0 {
                module.vmp_apply_dft(tmp_b_dft, a, vmp_pmat, buf);
            } else {
                module.vmp_apply_dft_to_dft_inplace(tmp_b_dft, vmp_pmat, buf);
            }
        });

        let mut tmp_b_big: VecZnxBig = tmp_b_dft.as_vec_znx_big();
        module.vec_znx_idft_tmp_a(&mut tmp_b_big, tmp_b_dft, b.cols());
        module.vec_znx_big_normalize(log_base2k, b, &tmp_b_big, buf);
    }

    pub fn product_inplace(
        &self,
        module: &Module,
        log_base2k: usize,
        a: &mut VecZnx,
        tmp_a_dft: &mut VecZnxDft,
        buf: &mut [u8],
    ) {
        self.value.iter().enumerate().for_each(|(i, vmp_pmat)| {
            if i == 0 {
                module.vmp_apply_dft(tmp_a_dft, a, vmp_pmat, buf);
            } else {
                module.vmp_apply_dft_to_dft_inplace(tmp_a_dft, vmp_pmat, buf);
            }
        });
        let mut tmp_b_big: VecZnxBig = tmp_a_dft.as_vec_znx_big();
        module.vec_znx_idft_tmp_a(&mut tmp_b_big, tmp_a_dft, a.cols());
        module.vec_znx_big_normalize(log_base2k, a, &tmp_b_big, buf);
    }
}
