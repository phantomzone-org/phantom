use crate::gadget::Gadget;
use math::modulus::WordOps;
use math::poly::Poly;
use math::ring::Ring;

pub struct Address {
    pub log_n_decomp: usize,
    pub gadget_matrix: Vec<Vec<Gadget<Poly<u64>>>>,
}

impl Address {
    pub fn new(ring: &Ring<u64>, log_base_gadget: usize, log_n_decomp: usize, size: usize) -> Self {
        let log_n: usize = ring.log_n();
        let mut gadget_matrix: Vec<Vec<Gadget<Poly<u64>>>> = Vec::new();
        let dims_n: usize = (size.log2() + log_n - 1) / log_n;
        let dims_n_decomp: usize = (log_n + log_n_decomp - 1) / log_n_decomp;

        (0..dims_n).for_each(|_| {
            let mut gadget_vec: Vec<Gadget<Poly<u64>>> = Vec::new();

            (0..dims_n_decomp).for_each(|_| gadget_vec.push(Gadget::new(&ring, log_base_gadget)));

            gadget_matrix.push(gadget_vec)
        });
        Self {
            gadget_matrix: gadget_matrix,
            log_n_decomp: log_n_decomp,
        }
    }

    pub fn log_base(&self) -> usize {
        self.gadget_matrix[0][0].log_base
    }

    pub fn dims_n(&self) -> usize {
        self.gadget_matrix.len()
    }

    pub fn dims_n_decomp(&self) -> usize {
        self.gadget_matrix[0].len()
    }

    pub fn set(&mut self, ring: &Ring<u64>, idx: usize) {
        let log_n: usize = ring.log_n();

        //assert!(idx > (1<<log_base * self.0.len()) == 0, "invalid idx: idx={} > {}*{}={}", idx, 1<<log_base, self.0.len(), 1<<(log_base*self.0.len()));

        let mask_log_n: usize = (1 << log_n) - 1;
        let mask_log_n_decomp: usize = (1 << self.log_n_decomp) - 1;
        let mut remain: usize = idx as _;
        let n: usize = ring.n();
        let q: u64 = ring.modulus.q();
        let mut buf: Poly<u64> = ring.new_poly();

        // Decomposition mod N
        self.gadget_matrix.iter_mut().for_each(|gadget_vec| {
            let mut chunk: usize = remain & mask_log_n;

            // Sub decomposition mod B < N
            gadget_vec.iter_mut().for_each(|gadget| {
                let sub_chunk: usize = chunk & mask_log_n_decomp;

                if sub_chunk != 0 {
                    buf.0[n - sub_chunk] = q - 1; // (X^i)^-1 = X^{2n-i} = -X^{n-i}
                } else {
                    buf.0[0] = 1;
                }

                gadget.encode(ring, &mut buf);

                if sub_chunk != 0 {
                    buf.0[n - sub_chunk] = 0;
                } else {
                    buf.0[0] = 0;
                }

                chunk >>= self.log_n_decomp;
            });

            remain >>= log_n;
        });
    }
}
