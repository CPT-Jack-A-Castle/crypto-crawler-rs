pub use crypto_market_type::MarketType;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

use super::utils::http_get;

lazy_static! {
    static ref LINEAR_CONTRACT_VALUES: HashMap<String, f64> = {
        // offline data, in case the network is down
        let mut m: HashMap<String, f64> = vec![
            ("1INCH/USDT", 1f64),
            ("AAVE/USDT", 0.01f64),
            ("ADA/USDT", 10f64),
            ("ALGO/USDT", 1f64),
            ("ATOM/USDT", 0.1f64),
            ("AVAX/USDT", 0.1f64),
            ("BAND/USDT", 0.1f64),
            ("BAT/USDT", 1f64),
            ("BCH/USDT", 0.01f64),
            ("BNB/USDT", 0.01f64),
            ("BSV/USDT", 0.01f64),
            ("BTC/USDT", 0.001f64),
            ("BTT/USDT", 1000f64),
            ("CHZ/USDT", 1f64),
            ("COMP/USDT", 0.01f64),
            ("CRV/USDT", 1f64),
            ("DASH/USDT", 0.01f64),
            ("DENT/USDT", 100f64),
            ("DGB/USDT", 10f64),
            ("DOGE/USDT", 100f64),
            ("DOT/USDT", 1f64),
            ("ENJ/USDT", 1f64),
            ("EOS/USDT", 1f64),
            ("ETC/USDT", 0.1f64),
            ("ETH/USDT", 0.01f64),
            ("FIL/USDT", 0.1f64),
            ("FTM/USDT", 1f64),
            ("GRT/USDT", 1f64),
            ("ICP/USDT", 0.01f64),
            ("IOST/USDT", 100f64),
            ("KSM/USDT", 0.01f64),
            ("LINK/USDT", 0.1f64),
            ("LTC/USDT", 0.1f64),
            ("LUNA/USDT", 1f64),
            ("MANA/USDT", 1f64),
            ("MATIC/USDT", 10f64),
            ("MIR/USDT", 0.1f64),
            ("MKR/USDT", 0.001f64),
            ("NEO/USDT", 0.1f64),
            ("OCEAN/USDT", 1f64),
            ("ONT/USDT", 1f64),
            ("QTUM/USDT", 0.1f64),
            ("RVN/USDT", 10f64),
            ("SHIB/USDT", 100000f64),
            ("SNX/USDT", 0.1f64),
            ("SOL/USDT", 0.1f64),
            ("SUSHI/USDT", 1f64),
            ("SXP/USDT", 1f64),
            ("THETA/USDT", 0.1f64),
            ("TRX/USDT", 100f64),
            ("UNI/USDT", 1f64),
            ("VET/USDT", 100f64),
            ("WAVES/USDT", 0.1f64),
            ("XEM/USDT", 1f64),
            ("XLM/USDT", 10f64),
            ("XMR/USDT", 0.01f64),
            ("XRP/USDT", 10f64),
            ("XTZ/USDT", 1f64),
            ("YFI/USDT", 0.0001f64),
            ("ZEC/USDT", 0.01f64),
        ]
        .into_iter()
        .map(|x| (x.0.to_string(), x.1))
        .collect();

        let from_online = fetch_linear_multipliers();
        for (pair, contract_value) in &from_online {
            m.insert(pair.clone(), *contract_value);
        }

        m
    };
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct SwapMarket {
    symbol: String,
    multiplier: f64,
    isInverse: bool,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct ResponseMsg {
    code: String,
    data: Vec<SwapMarket>,
}

// get the multiplier field from linear markets
fn fetch_linear_multipliers() -> BTreeMap<String, f64> {
    let mut mapping: BTreeMap<String, f64> = BTreeMap::new();

    let txt = http_get("https://api-futures.kucoin.com/api/v1/contracts/active").unwrap();
    let resp = serde_json::from_str::<ResponseMsg>(&txt).unwrap();
    for swap_market in resp.data.iter().filter(|x| !x.isInverse) {
        mapping.insert(
            crypto_pair::normalize_pair(&swap_market.symbol, "kucoin").unwrap(),
            swap_market.multiplier,
        );
    }

    mapping
}

pub(crate) fn get_contract_value(market_type: MarketType, pair: &str) -> Option<f64> {
    match market_type {
        MarketType::InverseSwap | MarketType::InverseFuture => Some(1.0),
        MarketType::LinearSwap => Some(LINEAR_CONTRACT_VALUES[pair]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::fetch_linear_multipliers;

    #[test]
    fn linear() {
        let mapping = fetch_linear_multipliers();
        for (pair, contract_value) in &mapping {
            println!("(\"{}\", {}f64),", pair, contract_value);
        }
    }
}
