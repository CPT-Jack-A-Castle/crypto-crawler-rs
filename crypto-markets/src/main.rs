use crypto_markets::fetch_markets;
use crypto_markets::MarketType;
use std::{env, str::FromStr};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: crypto-markets <exchange> <market_type>");
        return;
    }

    let exchange: &str = &args[1];
    let market_type = MarketType::from_str(&args[2]).unwrap();

    let markets = fetch_markets(exchange, market_type);
    println!("{}", serde_json::to_string_pretty(&markets).unwrap());
}
