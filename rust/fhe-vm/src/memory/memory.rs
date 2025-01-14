use math::modulus::montgomery::Montgomery;
use math::poly::Poly;
use math::ring::Ring;
use math::ring::impl_u64::packing::StreamRepacker;
use math::modulus::{ONCE, WordOps};

pub struct Memory(pub Vec<Poly<u64>>);

impl Memory{
    pub fn read(&self, ring: &Ring<u64>, idx: &Vec<Poly<u64>>) -> u64{

        let mut result: Vec<Poly<u64>> = self.pack_stream(ring, &self.0, &idx[0]);

        idx[1..].iter().for_each(|idx_i|{
            result = self.pack(ring, &mut result, idx_i);
        });
            
        ring.intt_inplace::<false>(&mut result[0]);
        
        result[0].0[0]
    }

    fn pack_stream(&self, ring: &Ring<u64>, data: &Vec<Poly<u64>>, idx: &Poly<Montgomery<u64>>) -> Vec<Poly<u64>>{

        let log_n: usize = ring.log_n();
        let mut packer: StreamRepacker = StreamRepacker::new(ring);
        let mut buf: Poly<u64> = ring.new_poly();

        for chunk in data.chunks(ring.n()){

            for i in 0..ring.n(){

                let i_rev: usize = i.reverse_bits_msb(log_n as u32);
            
                if i_rev < chunk.len(){
                    ring.a_mul_b_montgomery_into_c::<ONCE>(&chunk[i_rev], idx, &mut buf);
                    packer.add::<true>(ring, Some(&buf))
                }else{
                    packer.add::<true>(ring, None)
                }
            }
        }

        packer.flush::<true>(ring);
        return packer.results
    }

    fn pack(&self, ring: &Ring<u64>, data: &mut Vec<Poly<u64>>, idx: &Poly<Montgomery<u64>>) -> Vec<Poly<u64>>{

        let n: usize = ring.n();

        let mut results: Vec<Poly<u64>> = Vec::new();

        for chunk in data.chunks_mut(n){

            let mut result: Vec<Option<&mut Poly<u64>>> = Vec::with_capacity(n);
            result.resize_with(n, || None);

            chunk.iter_mut().enumerate().for_each(|(i, poly)|{
                ring.a_mul_b_montgomery_into_a::<ONCE>(idx, poly);
                result[i] = Some(poly)
            });

            ring.pack::<true, true>(&mut result, 0);

            if let Some(poly) = result[0].as_deref(){
                results.push(poly.clone())
            }
        }

        results
    }
}

