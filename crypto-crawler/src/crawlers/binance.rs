use core::panic;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use super::utils::{check_args, fetch_symbols_retry};
use crate::{msg::Message, MessageType};
use crypto_markets::MarketType;
use crypto_ws_client::*;
use log::*;

const EXCHANGE_NAME: &str = "binance";

// A single connection can listen to a maximum of 200 streams.
// see <https://binance-docs.github.io/apidocs/futures/en/#websocket-market-streams>
const MAX_SUBSCRIPTIONS_PER_CONNECTION: usize = 200;

#[rustfmt::skip]
gen_crawl_event!(crawl_trade_spot, BinanceSpotWSClient, MessageType::Trade, subscribe_trade);
#[rustfmt::skip]
gen_crawl_event!(crawl_trade_inverse, BinanceInverseWSClient, MessageType::Trade, subscribe_trade);
#[rustfmt::skip]
gen_crawl_event!(crawl_trade_linear, BinanceLinearWSClient, MessageType::Trade, subscribe_trade);
#[rustfmt::skip]
gen_crawl_event!(crawl_trade_linear_option, BinanceOptionWSClient, MessageType::Trade, subscribe_trade);

#[rustfmt::skip]
gen_crawl_event!(crawl_l2_event_spot, BinanceSpotWSClient, MessageType::L2Event, subscribe_orderbook);
#[rustfmt::skip]
gen_crawl_event!(crawl_l2_event_inverse, BinanceInverseWSClient, MessageType::L2Event, subscribe_orderbook);
#[rustfmt::skip]
gen_crawl_event!(crawl_l2_event_linear, BinanceLinearWSClient, MessageType::L2Event, subscribe_orderbook);
#[rustfmt::skip]
gen_crawl_event!(crawl_l2_event_linear_option, BinanceOptionWSClient, MessageType::L2Event, subscribe_orderbook);

#[rustfmt::skip]
gen_crawl_event!(crawl_ticker_spot, BinanceSpotWSClient, MessageType::Ticker, subscribe_ticker);
#[rustfmt::skip]
gen_crawl_event!(crawl_ticker_inverse, BinanceInverseWSClient, MessageType::Ticker, subscribe_ticker);
#[rustfmt::skip]
gen_crawl_event!(crawl_ticker_linear, BinanceLinearWSClient, MessageType::Ticker, subscribe_ticker);

pub(crate) fn crawl_trade(
    market_type: MarketType,
    symbols: Option<&[String]>,
    on_msg: Arc<Mutex<dyn FnMut(Message) + 'static + Send>>,
    duration: Option<u64>,
) -> Option<std::thread::JoinHandle<()>> {
    match market_type {
        MarketType::Spot => crawl_trade_spot(market_type, symbols, on_msg, duration),
        MarketType::InverseFuture | MarketType::InverseSwap => {
            crawl_trade_inverse(market_type, symbols, on_msg, duration)
        }
        MarketType::LinearFuture | MarketType::LinearSwap => {
            crawl_trade_linear(market_type, symbols, on_msg, duration)
        }
        MarketType::EuropeanOption => {
            if symbols.is_none() || symbols.unwrap().is_empty() {
                let on_msg_ext = Arc::new(Mutex::new(move |msg: String| {
                    let message = Message::new(
                        EXCHANGE_NAME.to_string(),
                        market_type,
                        MessageType::Trade,
                        msg,
                    );
                    (on_msg.lock().unwrap())(message);
                }));

                let channels: Vec<String> = vec![
                    "BTCUSDT_C@TRADE_ALL".to_string(),
                    "BTCUSDT_P@TRADE_ALL".to_string(),
                ];

                let ws_client = BinanceOptionWSClient::new(on_msg_ext, None);
                ws_client.subscribe(&channels);
                ws_client.run(duration);
                None
            } else {
                crawl_trade_linear_option(market_type, symbols, on_msg, duration)
            }
        }
        _ => panic!("Binance does NOT have the {} market type", market_type),
    }
}

pub(crate) fn crawl_l2_event(
    market_type: MarketType,
    symbols: Option<&[String]>,
    on_msg: Arc<Mutex<dyn FnMut(Message) + 'static + Send>>,
    duration: Option<u64>,
) -> Option<std::thread::JoinHandle<()>> {
    match market_type {
        MarketType::Spot => crawl_l2_event_spot(market_type, symbols, on_msg, duration),
        MarketType::InverseFuture | MarketType::InverseSwap => {
            crawl_l2_event_inverse(market_type, symbols, on_msg, duration)
        }
        MarketType::LinearFuture | MarketType::LinearSwap => {
            crawl_l2_event_linear(market_type, symbols, on_msg, duration)
        }
        MarketType::EuropeanOption => {
            crawl_l2_event_linear_option(market_type, symbols, on_msg, duration)
        }
        _ => panic!("Binance does NOT have the {} market type", market_type),
    }
}

pub(crate) fn crawl_ticker(
    market_type: MarketType,
    symbols: Option<&[String]>,
    on_msg: Arc<Mutex<dyn FnMut(Message) + 'static + Send>>,
    duration: Option<u64>,
) -> Option<std::thread::JoinHandle<()>> {
    let on_msg_clone = on_msg.clone();
    let on_msg_ext = Arc::new(Mutex::new(move |msg: String| {
        let message = Message::new(
            EXCHANGE_NAME.to_string(),
            market_type,
            MessageType::Ticker,
            msg,
        );
        (on_msg_clone.lock().unwrap())(message);
    }));

    if symbols.is_none() || symbols.unwrap().is_empty() {
        let channels: Vec<String> = vec!["!ticker@arr".to_string()];

        match market_type {
            MarketType::Spot => {
                let ws_client = BinanceSpotWSClient::new(on_msg_ext, None);
                ws_client.subscribe(&channels);
                ws_client.run(duration);
            }
            MarketType::InverseFuture | MarketType::InverseSwap => {
                let ws_client = BinanceInverseWSClient::new(on_msg_ext, None);
                ws_client.subscribe(&channels);
                ws_client.run(duration);
            }
            MarketType::LinearFuture | MarketType::LinearSwap => {
                let ws_client = BinanceLinearWSClient::new(on_msg_ext, None);
                ws_client.subscribe(&channels);
                ws_client.run(duration);
            }
            _ => panic!(
                "Binance {} market does NOT have the ticker channel",
                market_type
            ),
        }
        None
    } else {
        match market_type {
            MarketType::Spot => crawl_ticker_spot(market_type, symbols, on_msg, duration),
            MarketType::InverseFuture | MarketType::InverseSwap => {
                crawl_ticker_inverse(market_type, symbols, on_msg, duration)
            }
            MarketType::LinearFuture | MarketType::LinearSwap => {
                crawl_ticker_linear(market_type, symbols, on_msg, duration)
            }
            _ => panic!(
                "Binance {} market does NOT have the ticker channel",
                market_type
            ),
        }
    }
}

pub(crate) fn crawl_funding_rate(
    market_type: MarketType,
    symbols: Option<&[String]>,
    on_msg: Arc<Mutex<dyn FnMut(Message) + 'static + Send>>,
    duration: Option<u64>,
) {
    let on_msg_ext = Arc::new(Mutex::new(move |msg: String| {
        let message = Message::new(
            EXCHANGE_NAME.to_string(),
            market_type,
            MessageType::FundingRate,
            msg,
        );
        (on_msg.lock().unwrap())(message);
    }));

    let channels: Vec<String> = if symbols.is_none() || symbols.unwrap().is_empty() {
        vec!["!markPrice@arr".to_string()]
    } else {
        symbols
            .unwrap()
            .iter()
            .map(|symbol| format!("{}@markPrice", symbol.to_lowercase()))
            .collect()
    };

    match market_type {
        MarketType::InverseSwap => {
            let ws_client = BinanceInverseWSClient::new(on_msg_ext, None);
            ws_client.subscribe(&channels);
            ws_client.run(duration);
        }
        MarketType::LinearSwap => {
            let ws_client = BinanceLinearWSClient::new(on_msg_ext, None);
            ws_client.subscribe(&channels);
            ws_client.run(duration);
        }
        _ => panic!("Binance {} does NOT have funding rates", market_type),
    }
}
