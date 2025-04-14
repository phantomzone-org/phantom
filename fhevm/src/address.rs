use base2k::{
    alloc_aligned, alloc_aligned_u8, Infos, Module, VecZnx, VecZnxBig, VecZnxBigOps, VecZnxDft,
    VecZnxDftOps, VmpPMat, VmpPMatOps,
};
use itertools::izip;

use crate::{
    decompose::{Base1D, Base2D},
    memory::{read_tmp_bytes, Memory},
};

pub struct Address {
    pub rows: usize,
    pub cols: usize,
    pub coordinates_lsh: Vec<Coordinate>,
    pub coordinates_rsh: Vec<Coordinate>,
    pub base_2d: Base2D,
}

impl Address {
    pub fn new(module: &Module, base_2d: &Base2D, rows: usize, cols: usize) -> Self {
        let mut coordinates_lsh: Vec<Coordinate> = Vec::new();
        let mut coordinates_rsh: Vec<Coordinate> = Vec::new();
        base_2d.0.iter().for_each(|base_1d| {
            coordinates_lsh.push(Coordinate::new(module, rows, cols, base_1d));
            coordinates_rsh.push(Coordinate::new(module, rows, cols, base_1d));
        });
        Self {
            rows: rows,
            cols: cols,
            coordinates_lsh: coordinates_lsh,
            coordinates_rsh: coordinates_rsh,
            base_2d: base_2d.clone(),
        }
    }

    pub fn rows(&self) -> usize {
        self.coordinates_rsh[0].value[0].rows()
    }

    pub fn cols(&self) -> usize {
        self.coordinates_rsh[0].value[0].cols()
    }

    pub fn n2(&self) -> usize {
        self.coordinates_rsh.len()
    }

    pub fn n1(&self, idx: usize) -> usize {
        assert!(idx < self.coordinates_lsh.len());
        self.coordinates_rsh[idx].value.len()
    }

    pub fn set(&mut self, module: &Module, idx: u32) {
        debug_assert!(self.base_2d.max() > idx as usize);

        let mut remain: usize = idx as _;

        izip!(
            self.coordinates_lsh.iter_mut(),
            self.coordinates_rsh.iter_mut(),
            self.base_2d.0.iter(),
        )
        .for_each(|(coordinate_lsh, coordinate_rsh, base_1d)| {
            let max: usize = base_1d.max();
            let k: usize = remain & (max - 1);
            coordinate_lsh.encode(module, -(k as i64));
            coordinate_rsh.encode(module, k as i64);
            remain /= max;
        })
    }

    pub fn at_lsh(&self, i: usize) -> &Coordinate {
        &self.coordinates_lsh[i]
    }

    pub fn at_rsh(&self, i: usize) -> &Coordinate {
        &self.coordinates_rsh[i]
    }

    pub fn max(&self) -> usize {
        self.base_2d.max()
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
    pub base_1d: Base1D,
}

impl Coordinate {
    pub fn new(module: &Module, rows: usize, cols: usize, base_1d: &Base1D) -> Self {
        let mut coordinates: Vec<VmpPMat> = Vec::new();
        base_1d
            .0
            .iter()
            .for_each(|_| coordinates.push(module.new_vmp_pmat(rows, cols)));
        Self {
            value: coordinates,
            base_1d: base_1d.clone(),
        }
    }

    pub fn n1(&self) -> usize {
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
        let gap: usize = self.base_1d.gap(module.log_n());

        let mut tot_base: u8 = 0;
        izip!(self.value.iter_mut(), self.base_1d.0.iter()).for_each(|(vmp_pmat, base)| {
            let mask: usize = (1 << base) - 1;

            let chunk: usize = ((remain & mask) << tot_base) * gap;

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
