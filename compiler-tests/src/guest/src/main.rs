#![cfg_attr(target_arch = "riscv32", no_std, no_main)]
use core::{hint::black_box, panic::PanicInfo};
use macros::entry;
use serde::{Deserialize, Serialize};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

extern crate alloc;
extern crate runtime;

#[derive(Serialize, Deserialize)]
struct Pool {
    t0: u32,
    t1: u32,
}

#[derive(Serialize, Deserialize)]
struct Output {
    pool: Pool,
    tout: u32,
}

#[entry]
fn main() {
    let (mut pool, inp, control): (Pool, u32, bool) = read_input!();

    if control == true {
        pool.t0 += inp;
        pool.t1 += inp;
    }

    let mut out = 0;
    if pool.t0 > inp {
        pool.t0 -= inp;
        out = inp;
    }

    let output = Output { pool, tout: out };

    output!(output);
}
