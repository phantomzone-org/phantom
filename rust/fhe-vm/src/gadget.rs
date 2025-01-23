use math::modulus::{WordOps, ONCE};
use math::poly::Poly;
use math::ring::Ring;

pub struct Gadget<O> {
    pub value: Vec<O>,
    pub log_base: usize,
}

impl Gadget<Poly<u64>> {
    pub fn new(ring: &Ring<u64>, log_base: usize) -> Self {
        let mut value: Vec<Poly<u64>> = Vec::new();
        let d: usize = (ring.modulus.q.log2() as usize + log_base - 1) / log_base;
        (0..d).for_each(|_| value.push(ring.new_poly()));
        Self {
            value: value,
            log_base: log_base,
        }
    }

    pub fn at(&self, i: usize) -> &Poly<u64> {
        &self.value[i]
    }

    pub fn at_mut(&mut self, i: usize) -> &mut Poly<u64> {
        &mut self.value[i]
    }

    pub fn ntt(&mut self, ring: &Ring<u64>) {
        self.value.iter_mut().for_each(|poly: &mut Poly<u64>| {
            ring.ntt_inplace::<false>(poly);
        });
    }

    pub fn intt(&mut self, ring: &Ring<u64>) {
        self.value.iter_mut().for_each(|poly: &mut Poly<u64>| {
            ring.intt_inplace::<false>(poly);
        });
    }

    pub fn encode(&mut self, ring: &Ring<u64>, value: &Poly<u64>) {
        self.value.iter_mut().enumerate().for_each(|(i, poly)| {
            ring.a_mul_b_scalar_into_c::<ONCE>(value, &(1 << (i * self.log_base) as u64), poly);
            ring.ntt_inplace::<true>(poly);
            ring.a_prepare_montgomery_into_a::<ONCE>(poly);
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
        ring.intt::<false>(a, buf0);
        self.product_core(ring, buf0, buf1, buf2, b);
    }

    pub fn product_inplace(
        &self,
        ring: &Ring<u64>,
        buf0: &mut Poly<u64>,
        buf1: &mut Poly<u64>,
        buf2: &mut Poly<u64>,
        a: &mut Poly<u64>,
    ) {
        let (a_intt, a_decomp, carry) = (buf0, buf1, buf2);
        ring.intt::<false>(a, a_intt);
        self.product_core(ring, a_intt, a_decomp, carry, a);
    }

    fn product_core(
        &self,
        ring: &Ring<u64>,
        a_intt: &Poly<u64>,
        a_decomp: &mut Poly<u64>,
        carry: &mut Poly<u64>,
        b: &mut Poly<u64>,
    ) {
        self.value.iter().enumerate().for_each(|(i, rgsw_poly)| {
            ring.a_ith_digit_signed_base_scalar_b_into_c::<false>(
                i,
                a_intt,
                &self.log_base,
                carry,
                a_decomp,
            );
            ring.ntt_inplace::<false>(a_decomp);
            if i == 0 {
                ring.a_mul_b_montgomery_into_c::<ONCE>(a_decomp, rgsw_poly, b);
            } else {
                ring.a_mul_b_montgomery_add_c_into_c::<ONCE, ONCE>(a_decomp, rgsw_poly, b);
            }
        });
    }
}
