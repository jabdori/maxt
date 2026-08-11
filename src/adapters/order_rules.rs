use rust_decimal::Decimal;
use serde_json::Value;

use crate::{
    Balance, Error, Market, MarketStatus, OrderAccount, OrderOption, OrderRules, OrderType, Result,
    Side, TimeInForce,
};

pub(super) fn parse(body: &Value, expected: &Market, native_symbol: &str) -> Result<OrderRules> {
    let market = object(body, "market")?;
    let returned_symbol = text(market, "id")?;
    if returned_symbol != native_symbol {
        return Err(Error::decode(format!(
            "order rules returned `{returned_symbol}`, expected `{native_symbol}`"
        )));
    }

    let buy = object(market, "bid")?;
    let sell = object(market, "ask")?;
    expect_asset(buy, &expected.quote, "market.bid")?;
    expect_asset(sell, &expected.base, "market.ask")?;
    let quote_account = account(object(body, "bid_account")?, &expected.quote, "bid_account")?;
    let base_account = account(object(body, "ask_account")?, &expected.base, "ask_account")?;
    let (buy_price_unit, sell_price_unit) = if expected.exchange == crate::Exchange::Bithumb {
        (
            optional_decimal(buy, "price_unit")?,
            optional_decimal(sell, "price_unit")?,
        )
    } else {
        (None, None)
    };

    Ok(OrderRules {
        market: expected.clone(),
        market_name: text(market, "name")?.to_string(),
        status: match text(market, "state")? {
            "active" => MarketStatus::Active,
            "paused" | "halted" => MarketStatus::Paused,
            "delisted" => MarketStatus::Delisted,
            _ => MarketStatus::Unknown,
        },
        buy_fee_rate: decimal(body, "bid_fee")?,
        sell_fee_rate: decimal(body, "ask_fee")?,
        maker_buy_fee_rate: decimal(body, "maker_bid_fee")?,
        maker_sell_fee_rate: decimal(body, "maker_ask_fee")?,
        sides: sides(market)?,
        buy_options: options(market, "bid_types")?,
        sell_options: options(market, "ask_types")?,
        buy_price_unit,
        sell_price_unit,
        minimum_buy_total: decimal(buy, "min_total")?,
        minimum_sell_total: decimal(sell, "min_total")?,
        maximum_total: decimal(market, "max_total")?,
        quote_account,
        base_account,
    })
}

fn object<'a>(value: &'a Value, field: &'static str) -> Result<&'a Value> {
    value
        .get(field)
        .filter(|value| value.is_object())
        .ok_or_else(|| Error::decode(format!("order rules `{field}` is not an object")))
}

fn text<'a>(value: &'a Value, field: &'static str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| Error::decode(format!("order rules `{field}` is not text")))
}

fn decimal(value: &Value, field: &'static str) -> Result<Decimal> {
    let raw = match value.get(field) {
        Some(Value::String(raw)) => raw.clone(),
        Some(Value::Number(raw)) => raw.to_string(),
        _ => {
            return Err(Error::decode(format!(
                "order rules `{field}` is not a number"
            )));
        }
    };
    crate::adapters::decimal::exact(&raw).map_err(|error| {
        Error::decode(format!(
            "order rules `{field}` is not an exact decimal `{raw}`: {error}"
        ))
    })
}

fn optional_decimal(value: &Value, field: &'static str) -> Result<Option<Decimal>> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => decimal(value, field).map(Some),
    }
}

fn expect_asset(value: &Value, expected_asset: &str, context: &'static str) -> Result<String> {
    let asset = text(value, "currency")?.to_ascii_uppercase();
    if asset != expected_asset {
        return Err(Error::decode(format!(
            "order rules `{context}.currency` uses `{asset}`, expected `{expected_asset}`"
        )));
    }
    Ok(asset)
}

fn account(value: &Value, expected_asset: &str, context: &'static str) -> Result<OrderAccount> {
    let asset = expect_asset(value, expected_asset, context)?;
    let average_buy_price_unit = value
        .get("unit_currency")
        .filter(|value| !value.is_null())
        .map(|value| {
            value
                .as_str()
                .map(str::to_ascii_uppercase)
                .ok_or_else(|| Error::decode("order rules `unit_currency` is not text"))
        })
        .transpose()?;
    Ok(OrderAccount {
        balance: Balance {
            asset,
            available: decimal(value, "balance")?,
            locked: decimal(value, "locked")?,
        },
        average_buy_price: decimal(value, "avg_buy_price")?,
        average_buy_price_modified: value
            .get("avg_buy_price_modified")
            .and_then(Value::as_bool)
            .ok_or_else(|| Error::decode("order rules `avg_buy_price_modified` is not boolean"))?,
        average_buy_price_unit,
    })
}

fn sides(value: &Value) -> Result<Vec<Side>> {
    value
        .get("order_sides")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::decode("order rules `order_sides` is not an array"))?
        .iter()
        .map(|value| match value.as_str() {
            Some("bid") => Ok(Side::Buy),
            Some("ask") => Ok(Side::Sell),
            Some(other) => Err(Error::decode(format!(
                "order rules contains unknown side `{other}`"
            ))),
            None => Err(Error::decode("order rules `order_sides` contains non-text")),
        })
        .collect()
}

fn options(value: &Value, field: &'static str) -> Result<Vec<OrderOption>> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| Error::decode(format!("order rules `{field}` is not an array")))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(order_option)
                .ok_or_else(|| Error::decode(format!("order rules `{field}` contains non-text")))
        })
        .collect()
}

fn order_option(provider_id: &str) -> OrderOption {
    let (order_type, time_in_force) = match provider_id {
        "limit" => (Some(OrderType::Limit), None),
        "market" | "price" => (Some(OrderType::Market), None),
        "best" => (Some(OrderType::Best), None),
        "limit_ioc" => (Some(OrderType::Limit), Some(TimeInForce::ImmediateOrCancel)),
        "limit_fok" => (Some(OrderType::Limit), Some(TimeInForce::FillOrKill)),
        "best_ioc" => (Some(OrderType::Best), Some(TimeInForce::ImmediateOrCancel)),
        "best_fok" => (Some(OrderType::Best), Some(TimeInForce::FillOrKill)),
        _ => (None, None),
    };
    OrderOption {
        provider_id: provider_id.to_string(),
        order_type,
        time_in_force,
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::*;
    use crate::Exchange;

    #[test]
    fn normalizes_known_options_and_preserves_future_values() {
        let mut body = serde_json::json!({
            "bid_fee": "0.0025",
            "ask_fee": "0.0025",
            "maker_bid_fee": "0.001",
            "maker_ask_fee": "0.001",
            "market": {
                "id": "KRW-BTC",
                "name": "BTC/KRW",
                "state": "active",
                "order_sides": ["ask", "bid"],
                "bid_types": ["limit", "best_ioc", "future_order"],
                "ask_types": ["market"],
                "bid": {"currency": "KRW", "price_unit": "0.1", "min_total": "5000"},
                "ask": {"currency": "BTC", "price_unit": "0.1", "min_total": "5000"},
                "max_total": "1000000000"
            },
            "bid_account": {
                "currency": "KRW", "balance": "10000", "locked": "0",
                "avg_buy_price": "0", "avg_buy_price_modified": false,
                "unit_currency": "KRW"
            },
            "ask_account": {
                "currency": "BTC", "balance": "1", "locked": "0.1",
                "avg_buy_price": "95000000", "avg_buy_price_modified": false,
                "unit_currency": "KRW"
            }
        });

        let rules = parse(
            &body,
            &Market::spot(Exchange::Upbit, "BTC", "KRW"),
            "KRW-BTC",
        )
        .expect("valid order rules");

        assert_eq!(rules.minimum_buy_total, Decimal::from(5_000));
        assert_eq!(rules.sides, vec![Side::Sell, Side::Buy]);
        assert_eq!(rules.base_account.balance.asset, "BTC");
        assert_eq!(rules.buy_options[1].order_type, Some(OrderType::Best));
        assert_eq!(
            rules.buy_options[1].time_in_force,
            Some(TimeInForce::ImmediateOrCancel)
        );
        assert_eq!(rules.buy_options[2].provider_id, "future_order");
        assert_eq!(rules.buy_options[2].order_type, None);
        assert_eq!(rules.buy_price_unit, None);

        let bithumb_rules = parse(
            &body,
            &Market::spot(Exchange::Bithumb, "BTC", "KRW"),
            "KRW-BTC",
        )
        .expect("valid Bithumb order rules");
        assert_eq!(bithumb_rules.buy_price_unit, Some(Decimal::new(1, 1)));
        assert_eq!(bithumb_rules.sell_price_unit, Some(Decimal::new(1, 1)));

        body["market"]["bid"]["currency"] = Value::String("USDT".to_owned());
        assert!(
            parse(
                &body,
                &Market::spot(Exchange::Upbit, "BTC", "KRW"),
                "KRW-BTC"
            )
            .is_err()
        );
    }
}
