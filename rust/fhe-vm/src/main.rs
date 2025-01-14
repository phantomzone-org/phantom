use math::ring::Ring;
use fhe_vm::memory::memory::Memory;


fn main() {

    let n: usize = 1 << 4;
    let nth_root: usize = n << 1;
    let q_base: u64 = 65537u64;
    let q_power: usize = 1usize;
    let ring: Ring<u64> = Ring::new(n, q_base, q_power);


    println!("Hello, world!");
}
