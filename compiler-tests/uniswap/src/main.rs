use compiler::{CompileOpts, Phantom};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{ops::Div, ptr};

fn to_u8_slice<T>(v: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts((v as *const T) as *const u8, core::mem::size_of::<T>()) }
}

fn from_u8_slice<T>(v: &[u8]) -> T {
    unsafe { ptr::read(v.as_ptr() as *const T) }.into()
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
struct Account {
    t0_balance: u32,
    t1_balance: u32,
}

#[derive(Debug, Clone)]
#[repr(C)]
struct Trade {
    /// Token amount to sell
    amount: u32,
    /// Trade direction. Set true to sell t0 for t1, false to sell t1 for t0
    direction: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
struct Pool {
    t0: u32,
    t1: u32,
}

#[derive(Debug)]
#[repr(C)]
struct Output {
    pool: Pool,
    account: Account,
}

#[derive(Debug)]
#[repr(C)]
struct Input {
    pool: Pool,
    trade: Trade,
    account: Account,
}

fn main() {
    let compiler = CompileOpts::new("guest");
    let elf_bytes = compiler.build();
    let pz = Phantom::from_elf(elf_bytes);

    let mut rng = StdRng::from_seed([0; 32]);
    let mut pool = Pool { t0: 100, t1: 500 };
    let trade = Trade {
        amount: 50,
        direction: true,
    };
    let account = Account {
        t0_balance: 60,
        t1_balance: 0,
    };

    let input = Input {
        pool: pool.clone(),
        trade: trade.clone(),
        account: account.clone(),
    };

    let max_cycles = 1000;
    // let max_cycles = 10; // For testing purposes

    let mut enc_vm = pz.encrypted_vm(to_u8_slice(&input), max_cycles);
    enc_vm.execute();

    // Init -> read input tape -> run -> read output tape
    let mut vm = pz.test_vm(max_cycles);
    vm.read_input_tape(to_u8_slice(&input));
    vm.execute();
    let output_tape = vm.output_tape();
    println!("Output tape={:?}", output_tape);
    assert_eq!(output_tape, enc_vm.output_tape());
    println!("Encrypted Tape and Test VM Tape are equal");

    // Check output
    let mut want_pool = pool;
    let mut want_account = account;
    let k = want_pool.t0.wrapping_mul(want_pool.t1);
    if trade.direction == true {
        let t1_out = want_pool.t1 - (k.div(want_pool.t0.wrapping_add(trade.amount)));

        want_pool.t1 -= t1_out;
        want_pool.t0 += trade.amount;

        want_account.t0_balance -= trade.amount;
        want_account.t1_balance += t1_out;
    } else {
        let t0_out = want_pool.t0 - (k.div(want_pool.t1.wrapping_add(trade.amount)));

        want_pool.t0 -= t0_out;
        want_pool.t1 += trade.amount;

        want_account.t1_balance -= trade.amount;
        want_account.t0_balance += t0_out;
    }

    let have_output = from_u8_slice::<Output>(&output_tape);
    let have_account = have_output.account;
    let have_pool = have_output.pool;

    assert_eq!(have_account, want_account);
    assert_eq!(have_pool, want_pool);
}
