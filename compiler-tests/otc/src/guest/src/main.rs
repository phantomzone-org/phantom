#![cfg_attr(target_arch = "riscv32", no_std, no_main)]
use core::panic::PanicInfo;
use macros::entry;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

extern crate alloc;
extern crate runtime;

#[repr(C)]
enum ClientType {
    RETAIL,
    INSTITUTIONAL,
    PRIME,
}

#[repr(C)]
struct ClientProfile {
    client_type: ClientType,
}

#[repr(C)]
struct MarketData {
    // Mid market price
    mid_price: f32,
    /// volume in past 24 hours
    window_volume: f32,
    /// sqrt(`BTC/USD` daily annualized volatility / annualized volatility)
    volatility_factor: f32,
}

#[repr(C)]
struct Trade {
    amount: f32,
}

#[repr(C)]
struct Quote {
    ask_price: f32,
    bid_price: f32,
}

#[repr(C)]
struct Output {
    quote: Quote,
}

#[repr(C)]
struct Input {
    market_data: MarketData,
    client: ClientProfile,
    trade: Trade,
}

#[no_mangle]
#[link_section = ".inpdata"]
static INPUT: [u8; core::mem::size_of::<Input>()] = [0u8; core::mem::size_of::<Input>()];

#[no_mangle]
#[link_section = ".outdata"]
// Use SyncUnsafeCell when `static mut` gets decpreated: https://github.com/rust-lang/rust/issues/95439
static mut OUTPUT: [u8; core::mem::size_of::<Output>()] = [0u8; core::mem::size_of::<Output>()];

#[inline]
fn base_spread(client_type: &ClientType) -> f32 {
    match client_type {
        ClientType::INSTITUTIONAL => 25.0,
        ClientType::RETAIL => 100.0,
        ClientType::PRIME => 10.0,
    }
}

#[inline]
/// Increase spread for large trade volume
fn size_adj(trade: &Trade, market_data: &MarketData) -> f32 {
    let vol_perc = trade.amount / market_data.window_volume;

    let mut size_factor = 1.0;
    if vol_perc <= 0.01 {
        // vol <= 1%
        size_factor = 1.0;
    } else if vol_perc <= 0.05 {
        // vol <= 5% = 1.5
        size_factor = 1.5;
    } else {
        // >= 5%
        size_factor = 2.5;
    }

    size_factor
}

#[inline]
/// Volume based discounts to Insititutions and Prime customer to maintain strong relationship
fn volume_disc(client: &ClientType, volume: f32) -> f32 {
    match client {
        ClientType::PRIME | ClientType::INSTITUTIONAL => {
            if volume > 10.0 {
                0.7 // 30 % discount
            } else if volume > 5.0 {
                0.9 // 10 % discount
            } else {
                1.0
            }
        }
        _ => 1.0,
    }
}

#[entry]
fn main() {
    // READ INPUT
    let mut input: Input =
        unsafe { core::ptr::read_volatile(((&INPUT) as *const u8) as *const Input) };

    let market_data = input.market_data;
    let client = input.client;
    let trade = input.trade;

    let base_spread = base_spread(&client.client_type);

    // adjust base spread as per other factors
    let size_adj = size_adj(&trade, &market_data);
    let vol_adj = volume_disc(&client.client_type, trade.amount);

    let final_spread = base_spread * size_adj * vol_adj * market_data.volatility_factor;
    let spread_usd = final_spread / 10000.0 * market_data.mid_price;
    let spread_half = spread_usd / 2.0;

    // Buy price
    let ask_price = market_data.mid_price + spread_half;
    // Sell price
    let bid_price = market_data.mid_price - spread_half;

    let quote = Quote {
        ask_price,
        bid_price,
    };

    // WRITE OUTPUT
    let output_str = Output { quote };
    unsafe {
        core::ptr::copy_nonoverlapping(
            (&output_str as *const Output) as *const u8,
            OUTPUT.as_mut_ptr(),
            core::mem::size_of::<Output>(),
        )
    };

    loop {}
}
