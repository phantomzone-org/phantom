use math::modulus::ONCE;
use math::poly::Poly;
use math::ring::Ring;

pub struct TestVector(pub Poly<u64>);

impl TestVector {
    pub fn new(ring: &Ring<u64>, f: Box<dyn Fn(usize) -> usize>) -> Self {
        let mut test_vector: Poly<u64> = ring.new_poly();
        let n: usize = ring.n();
        let q: u64 = ring.modulus.q();
        test_vector.0.iter_mut().enumerate().for_each(|(i, x)| {
            *x = ring.modulus.montgomery.reduce::<ONCE>(q - f(n - i) as u64);
        });
        Self { 0: test_vector }
    }

    pub fn n(&self) -> usize {
        self.0.n()
    }
}
