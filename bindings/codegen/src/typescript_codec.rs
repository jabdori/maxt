use maxt_bindings_common::schema::{Field, Schema, Type};

use crate::{HEADER, lower_camel};

pub(crate) fn render(schema: &Schema) -> String {
    let mut output = String::from(HEADER);
    output.push_str(
        r#"
import { InvalidRequestError, errorFromWire } from "../errors.js";
import * as Model from "../models.js";
import { StreamError, StreamEvent, type StreamItem } from "../stream.js";
import type { NativeOutcome } from "./api.js";
import type * as Wire from "./contract.js";

function identifier<T extends { readonly id: string }>(
  values: readonly T[], id: string, field: string,
): T {
  const value = values.find((candidate) => candidate.id === id);
  if (value === undefined) throw new InvalidRequestError(field, `unknown value \`${id}\``);
  return value;
}

export function unwrapOutcome<T>(outcome: NativeOutcome<T>): T {
  if (!outcome.ok) throw errorFromWire(outcome.error);
  return outcome.value;
}

"#,
    );

    for name in schema.models {
        let record = schema
            .records
            .iter()
            .find(|record| record.name == format!("{name}Wire"))
            .unwrap_or_else(|| panic!("model {name} has no wire record"));
        output.push_str(&render_model(name, &record.fields));
    }
    output.push_str(SPECIAL_CODECS);
    output
}

fn render_model(name: &str, fields: &[Field]) -> String {
    match name {
        "OrderRequest" => return ORDER_REQUEST_CODEC.to_owned(),
        "HistoryRequest" => return HISTORY_REQUEST_CODEC.to_owned(),
        "StreamConfig" => return STREAM_CONFIG_CODEC.to_owned(),
        _ => {}
    }
    let function = lower_camel(name);
    let arguments = fields
        .iter()
        .map(|field| from_expression(&field.ty, &format!("value.{}", field.name), field.name))
        .collect::<Vec<_>>()
        .join(", ");
    let values = fields
        .iter()
        .map(|field| {
            let property = snake_to_camel(field.name);
            format!(
                "    {}: {},\n",
                field.name,
                to_expression(&field.ty, &format!("value.{property}"))
            )
        })
        .collect::<String>();
    format!(
        "export function {function}FromWire(value: Wire.{name}Wire): Model.{name} {{\n  return new Model.{name}({arguments});\n}}\n\nexport function {function}ToWire(value: Model.{name}): Wire.{name}Wire {{\n  return {{\n{values}  }};\n}}\n\n"
    )
}

fn snake_to_camel(value: &str) -> String {
    let mut output = String::new();
    let mut uppercase = false;
    for character in value.chars() {
        if character == '_' {
            uppercase = true;
        } else if uppercase {
            output.push(character.to_ascii_uppercase());
            uppercase = false;
        } else {
            output.push(character);
        }
    }
    output
}

fn from_expression(ty: &Type, value: &str, field: &str) -> String {
    match ty {
        Type::String | Type::Boolean | Type::Number => value.to_owned(),
        Type::Decimal => format!("Model.Decimal.parse({value})"),
        Type::Timestamp => format!("Model.Timestamp.fromNanoseconds(BigInt({value}))"),
        Type::Identifier(name) => {
            if *name == "HyperliquidLedgerKind" {
                format!("Model.HyperliquidLedgerKind.other({value})")
            } else {
                format!("identifier(Model.{name}.values, {value}, \"{field}\")")
            }
        }
        Type::Named("MarketKindWire") => {
            format!("identifier(Model.MarketKind.values, {value}, \"{field}\")")
        }
        Type::Named(name) if name.ends_with("Wire") => {
            format!(
                "{}FromWire({value})",
                lower_camel(name.trim_end_matches("Wire"))
            )
        }
        Type::Named(_) => value.to_owned(),
        Type::Optional(inner) => format!(
            "{value} === null ? null : {}",
            from_expression(inner, value, field)
        ),
        Type::List(inner) => format!(
            "{value}.map((item) => {})",
            from_expression(inner, "item", field)
        ),
        Type::Tuple(items) => format!(
            "[{}]",
            items
                .iter()
                .enumerate()
                .map(|(index, item)| from_expression(item, &format!("{value}[{index}]"), field))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn to_expression(ty: &Type, value: &str) -> String {
    match ty {
        Type::String | Type::Boolean | Type::Number => value.to_owned(),
        Type::Decimal => format!("{value}.toString()"),
        Type::Timestamp => format!("{value}.nanosecondsSinceEpoch.toString()"),
        Type::Identifier(_) => format!("{value}.id"),
        Type::Named("MarketKindWire") => {
            format!("{value} === Model.MarketKind.Spot ? \"spot\" : \"perpetual\"")
        }
        Type::Named(name) if name.ends_with("Wire") => {
            format!(
                "{}ToWire({value})",
                lower_camel(name.trim_end_matches("Wire"))
            )
        }
        Type::Named(_) => value.to_owned(),
        Type::Optional(inner) => {
            format!("{value} === null ? null : {}", to_expression(inner, value))
        }
        Type::List(inner) => format!("{value}.map((item) => {})", to_expression(inner, "item")),
        Type::Tuple(items) => format!(
            "[{}]",
            items
                .iter()
                .enumerate()
                .map(|(index, item)| to_expression(item, &format!("{value}[{index}]")))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

const ORDER_REQUEST_CODEC: &str = r#"export function orderRequestFromWire(value: Wire.OrderRequestWire): Model.OrderRequest {
  const size = value.size.kind === "base"
    ? Model.Size.base(Model.Decimal.parse(value.size.value))
    : Model.Size.quote(Model.Decimal.parse(value.size.value));
  const options = {
    timeInForce: value.time_in_force === null
      ? null
      : identifier(Model.TimeInForce.values, value.time_in_force, "time_in_force"),
    reduceOnly: value.reduce_only,
  };
  const market = marketFromWire(value.market);
  const side = identifier(Model.Side.values, value.side, "side");
  if (value.order_type === Model.OrderType.Market.id) {
    return Model.OrderRequest.market(market, side, size, options);
  }
  if (value.order_type === Model.OrderType.Limit.id && value.price !== null) {
    return Model.OrderRequest.limit(market, side, size, Model.Decimal.parse(value.price), options);
  }
  throw new InvalidRequestError("order_type", "invalid order request");
}

export function orderRequestToWire(value: Model.OrderRequest): Wire.OrderRequestWire {
  return {
    market: marketToWire(value.market), side: value.side.id, order_type: value.orderType.id,
    size: {
      kind: value.size.kind === Model.SizeKind.Base ? "base" : "quote",
      value: value.size.value.toString(),
    },
    price: value.price?.toString() ?? null, time_in_force: value.timeInForce?.id ?? null,
    reduce_only: value.reduceOnly,
  };
}

"#;

const STREAM_CONFIG_CODEC: &str = r#"export function streamConfigFromWire(value: Wire.StreamConfigWire): Model.StreamConfig {
  return new Model.StreamConfig({
    maxReconnectAttempts: value.max_reconnect_attempts,
    initialReconnectDelayMs: Number(value.initial_reconnect_delay_ms),
    maxReconnectDelayMs: Number(value.max_reconnect_delay_ms),
    idleTimeoutMs: Number(value.idle_timeout_ms),
    bufferSize: Number(value.buffer_size),
    overflow: identifier(Model.Overflow.values, value.overflow, "overflow"),
  });
}

export function streamConfigToWire(value: Model.StreamConfig): Wire.StreamConfigWire {
  return {
    max_reconnect_attempts: value.maxReconnectAttempts,
    initial_reconnect_delay_ms: value.initialReconnectDelayMs.toString(),
    max_reconnect_delay_ms: value.maxReconnectDelayMs.toString(),
    idle_timeout_ms: value.idleTimeoutMs.toString(),
    buffer_size: value.bufferSize.toString(), overflow: value.overflow.id,
  };
}

"#;

const HISTORY_REQUEST_CODEC: &str = r#"export function historyRequestFromWire(value: Wire.HistoryRequestWire): Model.HistoryRequest {
  return new Model.HistoryRequest(
    marketFromWire(value.market),
    value.from === null ? null : Model.Timestamp.fromNanoseconds(BigInt(value.from)),
    value.to === null ? null : Model.Timestamp.fromNanoseconds(BigInt(value.to)),
    value.cursor === null ? null : new Model.Cursor(value.cursor),
    value.limit,
  );
}

export function historyRequestToWire(value: Model.HistoryRequest): Wire.HistoryRequestWire {
  return {
    market: marketToWire(value.market),
    from: value.from?.nanosecondsSinceEpoch.toString() ?? null,
    to: value.to?.nanosecondsSinceEpoch.toString() ?? null,
    cursor: value.cursor?.value ?? null,
    limit: value.limit,
  };
}

"#;

const SPECIAL_CODECS: &str = r#"export function sizeFromWire(value: Wire.SizeWire): Model.Size {
  return value.kind === "base"
    ? Model.Size.base(Model.Decimal.parse(value.value))
    : Model.Size.quote(Model.Decimal.parse(value.value));
}

export function sizeToWire(value: Model.Size): Wire.SizeWire {
  return {
    kind: value.kind === Model.SizeKind.Base ? "base" : "quote",
    value: value.value.toString(),
  };
}

export function feedFromWire(value: Wire.FeedWire): Model.Feed {
  switch (value.kind) {
    case "trades": return Model.Feed.Trades;
    case "order_book": return Model.Feed.OrderBook;
    case "ticker": return Model.Feed.Ticker;
    case "candles": return Model.Feed.candles(identifier(Model.Interval.values, value.interval, "interval"));
  }
}

export function feedToWire(value: Model.Feed): Wire.FeedWire {
  if (value.kind !== "candles") return { kind: value.kind };
  if (value.interval === null) throw new InvalidRequestError("feed.interval", "candle feed requires an interval");
  return { kind: "candles", interval: value.interval.id };
}

export function marketEventFromWire(value: Wire.MarketEventWire): Model.MarketEvent {
  switch (value.kind) {
    case "trade": return { kind: "trade", trade: tradeFromWire(value.trade) };
    case "order_book": return { kind: "order_book", orderBook: orderBookFromWire(value.order_book) };
    case "ticker": return { kind: "ticker", ticker: tickerFromWire(value.ticker) };
    case "candle": return { kind: "candle", candle: candleFromWire(value.candle) };
    case "reconnected": return { kind: "reconnected" };
  }
}

export function marketEventToWire(value: Model.MarketEvent): Wire.MarketEventWire {
  switch (value.kind) {
    case "trade": return { kind: "trade", trade: tradeToWire(value.trade) };
    case "order_book": return { kind: "order_book", order_book: orderBookToWire(value.orderBook) };
    case "ticker": return { kind: "ticker", ticker: tickerToWire(value.ticker) };
    case "candle": return { kind: "candle", candle: candleToWire(value.candle) };
    case "reconnected": return { kind: "reconnected" };
  }
}

export function accountEventFromWire(value: Wire.AccountEventWire): Model.AccountEvent {
  switch (value.kind) {
    case "balance": return { kind: "balance", balance: balanceFromWire(value.balance) };
    case "order": return { kind: "order", order: orderFromWire(value.order) };
    case "reconnected": return { kind: "reconnected" };
  }
}

export function accountEventToWire(value: Model.AccountEvent): Wire.AccountEventWire {
  switch (value.kind) {
    case "balance": return { kind: "balance", balance: balanceToWire(value.balance) };
    case "order": return { kind: "order", order: orderToWire(value.order) };
    case "reconnected": return { kind: "reconnected" };
  }
}

export function marketStreamItemFromWire(value: Wire.MarketStreamItemWire): StreamItem<Model.MarketEvent> {
  return value.kind === "error"
    ? new StreamError(errorFromWire(value.error))
    : new StreamEvent(marketEventFromWire(value.event));
}

export function accountStreamItemFromWire(value: Wire.AccountStreamItemWire): StreamItem<Model.AccountEvent> {
  return value.kind === "error"
    ? new StreamError(errorFromWire(value.error))
    : new StreamEvent(accountEventFromWire(value.event));
}

export function pageFromWire<T, W>(value: Wire.PageWire<W>, decode: (wire: W) => T): Model.Page<T> {
  return new Model.Page(value.items.map(decode), value.next === null ? null : new Model.Cursor(value.next));
}

export const upbitRegionFromWire = (value: string): Model.UpbitRegion =>
  identifier(Model.UpbitRegion.values, value, "region");
export const binanceMarketFromWire = (value: string): Model.BinanceMarket =>
  identifier(Model.BinanceMarket.values, value, "venue");
export const upbitMarketEventPairFromWire = (
  value: readonly [Wire.MarketWire, Wire.UpbitMarketEventWire],
): readonly [Model.Market, Model.UpbitMarketEvent] =>
  [marketFromWire(value[0]), upbitMarketEventFromWire(value[1])];
export const marketStringPairFromWire = (
  value: readonly [Wire.MarketWire, string],
): readonly [Model.Market, string] => [marketFromWire(value[0]), value[1]];
export const bithumbMarketAlertPairFromWire = (
  value: readonly [Wire.MarketWire, Wire.BithumbMarketAlertWire],
): readonly [Model.Market, Model.BithumbMarketAlert] =>
  [marketFromWire(value[0]), bithumbMarketAlertFromWire(value[1])];

export function stringifyWire(value: unknown): string {
  return JSON.stringify(value);
}
"#;
