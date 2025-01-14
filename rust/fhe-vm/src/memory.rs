use math::modulus::montgomery::Montgomery;
use math::modulus::{WordOps, ONCE};
use math::poly::Poly;
use math::ring::impl_u64::packing::StreamRepacker;
use math::ring::Ring;

pub struct Memory(pub Vec<Poly<u64>>);
pub struct Index(pub Vec<Poly<u64>>);

impl Index {
    pub fn new(ring: &Ring<u64>, size: usize) -> Self {
        let n: usize = ring.n();
        let mut polys: Vec<Poly<u64>> = Vec::new();
        let dims: usize = (size + n - 1) / n;
        (0..dims).for_each(|_| polys.push(ring.new_poly()));
        Self { 0: polys }
    }

    pub fn set(&mut self, ring: &Ring<u64>, idx: usize) {
        let log_base: usize = ring.log_n();
        let mask: usize = (1 << log_base) - 1;
        let mut remain: usize = idx as _;
        let n: usize = ring.n();
        let minus_one: u64 = ring.modulus.montgomery.minus_one();

        self.0.iter_mut().for_each(|poly| {
            let chunk = remain & mask;

            poly.zero();

            if chunk != 0 {
                poly.0[n - chunk] = minus_one; // (X^i)^-1 = X^{2n-i} = -X^{n-i}
            } else {
                poly.0[0] = 1;
            }

            ring.ntt_inplace::<false>(poly);

            remain >>= log_base
        });
    }
}

impl Memory {
    pub fn new(ring: &Ring<u64>, data: &Vec<u64>) -> Self {
        let n: usize = ring.n();

        let mut polys: Vec<Poly<u64>> = Vec::new();

        for chunk in data.chunks(n) {
            let mut poly: Poly<u64> = ring.new_poly();
            poly.set(chunk);
            ring.ntt_inplace::<false>(&mut poly);
            polys.push(poly);
        }

        Self { 0: polys }
    }

    pub fn read(&self, ring: &Ring<u64>, idx: &Index) -> u64 {
        //println!("{:?}", idx.0);

        let mut result: Vec<Poly<u64>> = self.pack_stream(ring, &self.0, &idx.0[0]);

        idx.0[1..].iter().for_each(|idx_i| {
            result = self.pack(ring, &mut result, idx_i);
        });

        ring.intt_inplace::<false>(&mut result[0]);

        result[0].0[0]
    }

    fn pack_stream(
        &self,
        ring: &Ring<u64>,
        data: &Vec<Poly<u64>>,
        idx: &Poly<Montgomery<u64>>,
    ) -> Vec<Poly<u64>> {
        let log_n: usize = ring.log_n();
        let mut packer: StreamRepacker = StreamRepacker::new(ring);
        let mut buf: Poly<u64> = ring.new_poly();

        for chunk in data.chunks(ring.n()) {
            for i in 0..ring.n() {
                let i_rev: usize = i.reverse_bits_msb(log_n as u32);

                if i_rev < chunk.len() {
                    ring.a_mul_b_montgomery_into_c::<ONCE>(&chunk[i_rev], idx, &mut buf);
                    packer.add::<true>(ring, Some(&buf))
                } else {
                    packer.add::<true>(ring, None)
                }
            }
        }

        packer.flush::<true>(ring);
        return packer.results;
    }

    fn pack(
        &self,
        ring: &Ring<u64>,
        data: &mut Vec<Poly<u64>>,
        idx: &Poly<Montgomery<u64>>,
    ) -> Vec<Poly<u64>> {
        let log_n: usize = ring.log_n();
        let n: usize = 1 << log_n;

        let mut results: Vec<Poly<u64>> = Vec::new();

        for chunk in data.chunks_mut(n) {
            let mut result: Vec<Option<&mut Poly<u64>>> = Vec::with_capacity(n);
            result.resize_with(n, || None);

            chunk.iter_mut().enumerate().for_each(|(i, poly)| {
                ring.a_mul_b_montgomery_into_a::<ONCE>(idx, poly);
                result[i] = Some(poly)
            });

            ring.pack::<true, true>(&mut result, log_n);

            if let Some(poly) = result[0].as_deref() {
                results.push(poly.clone())
            }
        }

        results
    }
}
