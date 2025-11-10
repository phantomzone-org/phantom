#[repr(C)]
pub enum ClientType {
    RETAIL,
    INSTITUTIONAL,
    PRIME,
}

#[repr(C)]
pub struct ClientProfile {
    client_type: ClientType,
}

impl ClientProfile {
    pub fn new(client_type: ClientType) -> Self {
        Self { client_type }
    }
}

#[repr(C)]
pub struct MarketData {
    // mid point price of BTC/USD
    mid_price: f32,
    /// volume in past 24 hours
    window_volume: f32,
    /// sqrt(pair's volatility / annualized volatility)
    volatility_factor: f32,
}

impl MarketData {
    pub fn new(mid_price: f32, window_volume: f32, volatility_factor: f32) -> Self {
        MarketData {
            mid_price,
            window_volume,
            volatility_factor,
        }
    }
}

impl Default for MarketData {
    fn default() -> Self {
        MarketData {
            mid_price: 1000.0,
            window_volume: 100.0,
            volatility_factor: volatility_factor(),
        }
    }
}

#[repr(C)]
pub struct Trade {
    amount: f32,
}

impl Trade {
    pub fn new(amount: f32) -> Self {
        Trade { amount }
    }
}

#[repr(C)]
pub struct Quote {
    ask_price: f32,
    bid_price: f32,
}

impl Quote {
    pub fn ask_price(&self) -> f32 {
        self.ask_price
    }

    pub fn bid_price(&self) -> f32 {
        self.bid_price
    }
}

fn base_spread(client_type: &ClientType) -> f32 {
    match client_type {
        ClientType::INSTITUTIONAL => 25.0,
        ClientType::RETAIL => 100.0,
        ClientType::PRIME => 10.0,
    }
}

/// - increase spread for larger trades
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

/// give volume based discounts to FIs and Prime due to longer relationship
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

pub fn volatility_factor() -> f32 {
    let base_annualized_volatilty = 0.6f32; // for crypto markets
    let daily_annualized_volatility = 0.47; // btc daily volatility, calculated on 9th November 2025
    let volatility_ratio = daily_annualized_volatility / base_annualized_volatilty;
    volatility_ratio.sqrt()
}

pub fn quote(client: &ClientProfile, trade: &Trade, market_data: &MarketData) -> Quote {
    let base_spread = base_spread(&client.client_type);

    // adjust base spread as per other factors
    let size_adj = size_adj(trade, market_data);
    let vol_adj = volume_disc(&client.client_type, trade.amount);

    let final_spread = base_spread * size_adj * vol_adj * market_data.volatility_factor;
    let spread_usd = final_spread / 10000.0 * market_data.mid_price;
    let spread_half = spread_usd / 2.0;

    // Buy price
    let ask_price = market_data.mid_price + spread_half;
    // Sell price
    let bid_price = market_data.mid_price - spread_half;

    return Quote {
        ask_price,
        bid_price,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let client = ClientProfile {
            client_type: ClientType::PRIME,
        };
        let trade = Trade { amount: 5.0 };

        // Market data
        let market_data = MarketData {
            mid_price: 1000.0,
            window_volume: 100.0,
            volatility_factor: volatility_factor(),
        };

        let quote = quote(&client, &trade, &market_data);
        println!(
            "Ask price={}, Bid price={}",
            quote.ask_price, quote.bid_price
        );
    }
}
