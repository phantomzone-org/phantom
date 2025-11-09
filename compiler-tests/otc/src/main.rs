use std::ptr;

use compiler::{CompileOpts, Phantom};
use otc::{quote, ClientProfile, ClientType, MarketData, Quote, Trade};

fn to_u8_slice<T>(v: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts((v as *const T) as *const u8, core::mem::size_of::<T>()) }
}

fn from_u8_slice<T>(v: &[u8]) -> T {
    unsafe { ptr::read(v.as_ptr() as *const T) }.into()
}

#[repr(C)]
struct Input {
    market_data: MarketData,
    client: ClientProfile,
    trade: Trade,
}

#[repr(C)]
struct Output {
    quote: Quote,
}

fn main() {
    let compiler = CompileOpts::new("guest");
    let elf_bytes = compiler.build();
    let pz = Phantom::init(elf_bytes);

    let client = ClientProfile::new(ClientType::PRIME);
    let trade = Trade::new(5.0);
    let market_data = MarketData::default();
    let input = Input {
        market_data,
        client,
        trade,
    };

    // test vm
    let max_cycles = 9_000;
    let mut testvm = pz.test_vm(max_cycles);
    let testvm_input = to_u8_slice(&input);
    testvm.read_input_tape(&testvm_input);
    testvm.execute();
    let testvm_output = from_u8_slice::<Output>(&testvm.output_tape());

    let have_quote = testvm_output.quote;
    let want_quote = quote(&input.client, &input.trade, &input.market_data);

    println!(
        "Want ask price={}, Bid price={}",
        want_quote.ask_price(),
        want_quote.bid_price()
    );

    println!(
        "Have ask price={}, Bid price={}",
        have_quote.ask_price(),
        have_quote.bid_price()
    );

    println!("Hello, world!");
}
