#![cfg_attr(target_arch = "riscv32", no_std, no_main)]
use core::{ops::Div, panic::PanicInfo};
use macros::entry;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

extern crate alloc;
extern crate runtime;

#[repr(C)]
struct Pool {
    t0: u32,
    t1: u32,
}

#[repr(C)]
struct Output {
    pool: Pool,
    account: Account,
}

#[repr(C)]
struct Trade {
    /// Token amount to sell
    amount: u32,
    /// Trade direction. Set true to sell t0 for t1, false to sell t1 for t0
    direction: bool,
}

#[repr(C)]
struct Account {
    t0_balance: u32,
    t1_balance: u32,
}

#[repr(C)]
struct Input {
    pool: Pool,
    trade: Trade,
    account: Account,
}

#[no_mangle]
#[link_section = ".inpdata"]
static INPUT: [u8; core::mem::size_of::<Input>()] = [0u8; core::mem::size_of::<Input>()];

#[no_mangle]
#[link_section = ".outdata"]
// Use SyncUnsafeCell when `static mut` gets decpreated: https://github.com/rust-lang/rust/issues/95439
static mut OUTPUT: [u8; core::mem::size_of::<Output>()] = [0u8; core::mem::size_of::<Output>()];

#[entry]
fn main() {
    // READ INPUT
    let mut input: Input =
        unsafe { core::ptr::read_volatile(((&INPUT) as *const u8) as *const Input) };

    let mut pool = input.pool;
    let trade = input.trade;
    let mut account = input.account;

    let k = pool.t0.wrapping_mul(pool.t1);

    if trade.direction == true {
        assert!(account.t0_balance >= trade.amount);

        // Sell t0
        let t1_out = pool.t1 - (k.div(pool.t0.wrapping_add(trade.amount)));

        // update amount
        pool.t1 -= t1_out;
        pool.t0 += trade.amount;

        // update account
        account.t0_balance -= trade.amount;
        account.t1_balance += t1_out;
    } else {
        assert!(account.t1_balance >= trade.amount);

        // Sell t1
        let t0_out = pool.t0 - (k.div(pool.t1.wrapping_add(trade.amount)));

        // update pool
        pool.t0 -= t0_out;
        pool.t1 += trade.amount;

        // update account
        account.t1_balance -= trade.amount;
        account.t0_balance += t0_out;
    }

    // WRITE OUTPUT
    let output_str = Output { pool, account };
    unsafe {
        core::ptr::copy_nonoverlapping(
            (&output_str as *const Output) as *const u8,
            OUTPUT.as_mut_ptr(),
            core::mem::size_of::<Output>(),
        )
    };

    loop {}
}
