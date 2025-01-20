use math::modulus::montgomery::Montgomery;
use math::modulus::WordOps;
use math::poly::Poly;
use math::ring::Ring;

pub struct Address(pub Vec<Poly<u64>>);

impl Address {
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
        let minus_one: Montgomery<u64> = ring.modulus.montgomery.minus_one();
        let one: Montgomery<u64> = ring.modulus.montgomery.one();

        self.0.iter_mut().for_each(|poly| {
            let chunk = remain & mask;

            poly.zero();

            if chunk != 0 {
                poly.0[n - chunk] = minus_one; // (X^i)^-1 = X^{2n-i} = -X^{n-i}
            } else {
                poly.0[0] = one;
            }

            ring.ntt_inplace::<false>(poly);

            remain >>= log_base
        });
    }
}
