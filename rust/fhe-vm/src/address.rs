use crate::gadget::Gadget;
use math::modulus::WordOps;
use math::poly::Poly;
use math::ring::Ring;

pub struct Address(pub Vec<Gadget<Poly<u64>>>);

impl Address {
    pub fn new(ring: &Ring<u64>, log_base: usize, size: usize) -> Self {
        let log_n = ring.log_n();
        let mut polys: Vec<Gadget<Poly<u64>>> = Vec::new();
        let dims: usize = (size.log2() + log_n - 1) / log_n;
        (0..dims).for_each(|_| polys.push(Gadget::new(&ring, log_base)));
        Self { 0: polys }
    }

    pub fn log_base(&self) -> usize {
        self.0[0].log_base
    }

    pub fn set(&mut self, ring: &Ring<u64>, idx: usize) {
        let log_base: usize = ring.log_n();

        //assert!(idx > (1<<log_base * self.0.len()) == 0, "invalid idx: idx={} > {}*{}={}", idx, 1<<log_base, self.0.len(), 1<<(log_base*self.0.len()));

        let mask: usize = (1 << log_base) - 1;
        let mut remain: usize = idx as _;
        let n: usize = ring.n();
        let q: u64 = ring.modulus.q();
        let mut buf: Poly<u64> = ring.new_poly();

        self.0.iter_mut().for_each(|gadget| {
            let chunk = remain & mask;

            if chunk != 0 {
                buf.0[n - chunk] = q - 1; // (X^i)^-1 = X^{2n-i} = -X^{n-i}
            } else {
                buf.0[0] = 1;
            }

            gadget.encode(ring, &mut buf);

            if chunk != 0 {
                buf.0[n - chunk] = 0;
            } else {
                buf.0[0] = 0;
            }

            remain >>= log_base
        });
    }
}
