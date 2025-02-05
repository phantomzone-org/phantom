use base2k::{
    Infos, Matrix3D, Module, VecZnx, VecZnxBig, VecZnxDft, VecZnxOps, VmpPMat, VmpPMatOps,
};
use itertools::izip;

pub struct Address {
    pub log_n_decomp: usize,
    pub coordinates_lsh: Vec<Coordinate>,
    pub coordinates_rsh: Vec<Coordinate>,
}

impl Address {
    pub fn new(
        module: &Module,
        log_n_decomp: usize,
        size: usize,
        rows: usize,
        cols: usize,
    ) -> Self {
        let log_n: usize = module.log_n();
        let mut coordinates_lsh: Vec<Coordinate> = Vec::new();
        let mut coordinates_rsh: Vec<Coordinate> = Vec::new();
        let dims_n: usize =
            ((usize::BITS - (size - 1).leading_zeros()) as usize + log_n - 1) / log_n;
        let dims_n_decomp: usize = (log_n + log_n_decomp - 1) / log_n_decomp;
        (0..dims_n).for_each(|_| {
            coordinates_lsh.push(Coordinate::new(module, rows, cols, dims_n_decomp));
            coordinates_rsh.push(Coordinate::new(module, rows, cols, dims_n_decomp));
        });
        Self {
            coordinates_lsh: coordinates_lsh,
            coordinates_rsh: coordinates_rsh,
            log_n_decomp: log_n_decomp,
        }
    }

    //pub fn log_base(&self) -> usize {
    //    self.coordinates[0].0[0].log_base
    //}

    pub fn rows(&self) -> usize {
        self.coordinates_rsh[0].0[0].rows()
    }

    pub fn cols(&self) -> usize {
        self.coordinates_rsh[0].0[0].cols()
    }

    pub fn dims_n(&self) -> usize {
        self.coordinates_rsh.len()
    }

    pub fn dims_n_decomp(&self) -> usize {
        self.coordinates_rsh[0].0.len()
    }

    pub fn set(&mut self, module: &Module, idx: usize) {
        let log_n: usize = module.log_n();
        let mask_log_n: usize = (1 << log_n) - 1;
        let mut remain: usize = idx as _;

        izip!(
            self.coordinates_lsh.iter_mut(),
            self.coordinates_rsh.iter_mut()
        )
        .for_each(|(coordinate_lsh, coordinate_rsh)| {
            let k = remain & mask_log_n;
            coordinate_lsh.encode(module, -(k as i64), self.log_n_decomp);
            coordinate_rsh.encode(module, k as i64, self.log_n_decomp);
            remain >>= log_n;
        })
    }

    pub fn at_lsh(&self, i: usize) -> &Coordinate {
        &self.coordinates_lsh[i]
    }

    pub fn at_rsh(&self, i: usize) -> &Coordinate {
        &self.coordinates_rsh[i]
    }
}

pub struct Coordinate(Vec<VmpPMat>);

impl Coordinate {
    pub fn new(module: &Module, rows: usize, cols: usize, dims: usize) -> Self {
        let mut coordinates: Vec<VmpPMat> = Vec::new();
        (0..dims).for_each(|_| coordinates.push(module.new_vmp_pmat(rows, cols)));
        Self { 0: coordinates }
    }

    pub fn dims(&self) -> usize {
        self.0.len()
    }

    pub fn encode(&mut self, module: &Module, value: i64, log_base: usize) {
        let n: usize = module.n();
        let mask: usize = (1 << log_base) - 1;
        let rows: usize = self.0[0].rows();
        let cols: usize = self.0[0].cols();

        let sign: i64 = value.signum();
        let mut remain: usize = value.abs() as usize;

        let mut data_mat: Matrix3D<i64> = Matrix3D::new(rows, cols, module.n());
        let mut buf: Vec<u8> =
            vec![u8::default(); module.vmp_prepare_contiguous_tmp_bytes(rows, cols)];
        let mut buf_i64: Vec<i64> = vec![i64::default(); n];

        self.0.iter_mut().enumerate().for_each(|(i, vmp_pmat)| {
            let chunk: usize = (remain & mask) << (i * log_base);

            if sign < 0 && chunk != 0 {
                buf_i64[n - chunk] = -1; // (X^i)^-1 = X^{2n-i} = -X^{n-i}
            } else {
                buf_i64[chunk] = 1;
            }

            (0..data_mat.rows).for_each(|i| {
                data_mat.at_mut(i, i).copy_from_slice(&buf_i64);
            });

            module.vmp_prepare_contiguous(vmp_pmat, &data_mat.data, &mut buf);

            if sign < 0 && chunk != 0 {
                buf_i64[n - chunk] = 0;
            } else {
                buf_i64[chunk] = 0;
            }

            remain >>= log_base;
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
        self.0.iter().enumerate().for_each(|(i, vmp_pmat)| {
            if i == 0 {
                module.vmp_apply_dft(tmp_b_dft, a, vmp_pmat, buf);
            } else {
                module.vmp_apply_dft_to_dft_inplace(tmp_b_dft, vmp_pmat, buf);
            }
        });

        let mut tmp_b_big: VecZnxBig = tmp_b_dft.as_vec_znx_big();
        module.vec_znx_idft_tmp_a(&mut tmp_b_big, tmp_b_dft, b.limbs());
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
        self.0.iter().enumerate().for_each(|(i, vmp_pmat)| {
            if i == 0 {
                module.vmp_apply_dft(tmp_a_dft, a, vmp_pmat, buf);
            } else {
                module.vmp_apply_dft_to_dft_inplace(tmp_a_dft, vmp_pmat, buf);
            }
        });
        let mut tmp_b_big: VecZnxBig = tmp_a_dft.as_vec_znx_big();
        module.vec_znx_idft_tmp_a(&mut tmp_b_big, tmp_a_dft, a.limbs());
        module.vec_znx_big_normalize(log_base2k, a, &tmp_b_big, buf);
    }
}
