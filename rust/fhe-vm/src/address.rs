use crate::gadget::Gadget;
use itertools::izip;
use math::automorphism::AutoPerm;
use math::modulus::WordOps;
use math::poly::Poly;
use math::ring::Ring;

pub struct Address {
    pub log_n_decomp: usize,
    pub coordinates: Vec<Coordinate>,
}

impl Address {
    pub fn new(ring: &Ring<u64>, log_base_gadget: usize, log_n_decomp: usize, size: usize) -> Self {
        let log_n: usize = ring.log_n();
        let mut coordinates: Vec<Coordinate> = Vec::new();
        let dims_n: usize = (size.log2() + log_n - 1) / log_n;
        let dims_n_decomp: usize = (log_n + log_n_decomp - 1) / log_n_decomp;
        (0..dims_n)
            .for_each(|_| coordinates.push(Coordinate::new(ring, log_base_gadget, dims_n_decomp)));
        Self {
            coordinates: coordinates,
            log_n_decomp: log_n_decomp,
        }
    }

    pub fn log_base(&self) -> usize {
        self.coordinates[0].0[0].log_base
    }

    pub fn dims_n(&self) -> usize {
        self.coordinates.len()
    }

    pub fn dims_n_decomp(&self) -> usize {
        self.coordinates[0].0.len()
    }

    pub fn set(&mut self, ring: &Ring<u64>, idx: usize) {
        let log_n: usize = ring.log_n();
        let mask_log_n: usize = (1 << log_n) - 1;
        let mut remain: usize = idx as _;
        let mut buf: Poly<u64> = ring.new_poly();
        self.coordinates.iter_mut().for_each(|coordinate| {
            coordinate.encode(ring, remain & mask_log_n, self.log_n_decomp, &mut buf);
            remain >>= log_n;
        });
    }

    pub fn at(&self, i: usize) -> &Coordinate {
        &self.coordinates[i]
    }
}

pub struct Coordinate(Vec<Gadget<Poly<u64>>>);

impl Coordinate {
    pub fn new(ring: &Ring<u64>, log_base: usize, dims: usize) -> Self {
        let mut coordinates: Vec<Gadget<Poly<u64>>> = Vec::new();
        (0..dims).for_each(|_| coordinates.push(Gadget::new(&ring, log_base)));
        Self { 0: coordinates }
    }

    pub fn dims(&self) -> usize {
        self.0.len()
    }

    pub fn encode(&mut self, ring: &Ring<u64>, value: usize, log_base: usize, buf: &mut Poly<u64>) {
        let n: usize = ring.n();
        let mask: usize = (1 << log_base) - 1;
        let q: u64 = ring.modulus.q();

        let mut remain: usize = value;

        self.0.iter_mut().enumerate().for_each(|(i, gadget)| {
            let chunk: usize = (remain & mask) << (i * log_base);

            if chunk != 0 {
                buf.0[n - chunk] = q - 1; // (X^i)^-1 = X^{2n-i} = -X^{n-i}
            } else {
                buf.0[chunk] = 1;
            }

            gadget.encode(ring, buf);

            if chunk != 0 {
                buf.0[n - chunk] = 0;
            } else {
                buf.0[chunk] = 0;
            }

            remain >>= log_base;
        });
    }

    pub fn product(
        &self,
        ring: &Ring<u64>,
        a: &Poly<u64>,
        buf0: &mut Poly<u64>,
        buf1: &mut Poly<u64>,
        buf2: &mut Poly<u64>,
        b: &mut Poly<u64>,
    ) {
        self.0.iter().enumerate().for_each(|(i, gadget)| {
            if i == 0 {
                gadget.product(&ring, a, buf0, buf1, buf2, b);
            } else {
                gadget.product_inplace(&ring, buf0, buf1, buf2, b);
            }
        });
    }

    pub fn product_inplace(
        &self,
        ring: &Ring<u64>,
        buf0: &mut Poly<u64>,
        buf1: &mut Poly<u64>,
        buf2: &mut Poly<u64>,
        a: &mut Poly<u64>,
    ) {
        self.0.iter().for_each(|gadget| {
            gadget.product_inplace(&ring, buf0, buf1, buf2, a);
        });
    }

    pub fn reverse(&self, ring: &Ring<u64>, auto_perm: &AutoPerm, a: &mut Coordinate) {
        izip!(self.0.iter(), a.0.iter_mut()).for_each(|(a, b)| {
            izip!(a.value.iter(), b.value.iter_mut()).for_each(|(a_sub, b_sub)| {
                ring.a_apply_automorphism_from_perm_into_b::<true>(a_sub, auto_perm, b_sub);
            });
        });
    }
}
