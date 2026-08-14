use maxt_bindings_common::schema::{Field, Schema, TaggedUnion, Type};

use crate::typescript_contract::{HEADER, lower_camel};

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

function unsignedInteger(value: string, field: string): bigint {
  if (!/^\d+$/.test(value)) {
    throw new InvalidRequestError(field, "must be an unsigned decimal integer");
  }
  const parsed = BigInt(value);
  if (parsed > 18_446_744_073_709_551_615n) {
    throw new InvalidRequestError(field, "exceeds the Rust u64 range");
  }
  return parsed;
}

function safeUnsignedInteger(value: string, field: string): number {
  const parsed = unsignedInteger(value, field);
  if (parsed > BigInt(Number.MAX_SAFE_INTEGER)) {
    throw new InvalidRequestError(field, "exceeds the JavaScript safe integer range");
  }
  return Number(parsed);
}

export function checkedU32(value: number, field: string): number {
  if (!Number.isSafeInteger(value) || value < 0 || value > 4_294_967_295) {
    throw new InvalidRequestError(field, "must be a non-negative safe integer within the u32 range");
  }
  return value;
}

export function checkedOptionalU32(value: number | null, field: string): number | null {
  return value === null ? null : checkedU32(value, field);
}

function assertNever(value: never): never {
  throw new InvalidRequestError("kind", `unknown tagged union variant: ${String(value)}`);
}

export function unwrapOutcome<T>(outcome: NativeOutcome<T>): T {
  if (!outcome.ok) throw errorFromWire(outcome.error);
  return outcome.value;
}

"#,
    );

    for name in schema.models {
        if let Some(record) = schema
            .records
            .iter()
            .find(|record| record.name == format!("{name}Wire"))
        {
            output.push_str(&render_model(schema, name, &record.fields));
        } else if let Some(union) = schema
            .unions
            .iter()
            .find(|union| union.name == format!("{name}Wire"))
        {
            output.push_str(&render_union_model(schema, name, union));
        } else {
            panic!("model {name} has no wire record or tagged union");
        }
    }
    output.push_str(SPECIAL_CODECS);
    output
}

fn render_model(schema: &Schema, name: &str, fields: &[Field]) -> String {
    match name {
        "OrderRequest" => return ORDER_REQUEST_CODEC.to_owned(),
        "OrderHistoryRequest" => return ORDER_HISTORY_REQUEST_CODEC.to_owned(),
        "HistoryRequest" => return HISTORY_REQUEST_CODEC.to_owned(),
        "TransferHistoryRequest" => return TRANSFER_HISTORY_REQUEST_CODEC.to_owned(),
        "StreamConfig" => return STREAM_CONFIG_CODEC.to_owned(),
        _ => {}
    }
    let function = lower_camel(name);
    let allowed = fields
        .iter()
        .map(|field| format!("\"{}\"", snake_to_camel(field.name)))
        .collect::<Vec<_>>()
        .join(", ");
    let unexpected_checks = format!(
        "  for (const key of Object.keys(value)) {{\n    if (![{allowed}].includes(key)) {{\n      throw new InvalidRequestError(\"{function}.\" + key, \"{name} does not accept \" + key);\n    }}\n  }}\n"
    );
    let arguments = fields
        .iter()
        .map(|field| record_from_expression(schema, field, &format!("value.{}", field.name)))
        .collect::<Vec<_>>()
        .join(", ");
    let values = fields
        .iter()
        .map(|field| {
            let property = snake_to_camel(field.name);
            format!(
                "    {}: {},\n",
                field.name,
                record_to_expression(field, &format!("value.{property}"))
            )
        })
        .collect::<String>();
    format!(
        "export function {function}FromWire(value: Wire.{name}Wire): Model.{name} {{\n  return new Model.{name}({arguments});\n}}\n\nexport function {function}ToWire(value: Model.{name}): Wire.{name}Wire {{\n{unexpected_checks}  return {{\n{values}  }};\n}}\n\n"
    )
}

fn record_from_expression(schema: &Schema, field: &Field, value: &str) -> String {
    if field.name == "cursor" {
        return format!("{value} === null ? null : new Model.Cursor({value})");
    }
    from_expression(schema, &field.ty, value, field.name)
}

fn record_to_expression(field: &Field, value: &str) -> String {
    if field.name == "cursor" {
        return format!("{value} === null ? null : {value}.value");
    }
    to_expression(&field.ty, value)
}

fn render_union_model(schema: &Schema, name: &str, union: &TaggedUnion) -> String {
    let function = lower_camel(name);
    let from_arms = union
        .variants
        .iter()
        .map(|variant| {
            let fields = variant
                .fields
                .iter()
                .map(|field| {
                    format!(
                        ", {}: {}",
                        snake_to_camel(field.name),
                        from_expression(
                            schema,
                            &field.ty,
                            &format!("value.{}", field.name),
                            field.name,
                        )
                    )
                })
                .collect::<String>();
            format!(
                "    case \"{}\": return Object.freeze({{ kind: \"{}\"{fields} }});\n",
                variant.name, variant.name
            )
        })
        .collect::<String>();
    let to_arms = union
        .variants
        .iter()
        .map(|variant| {
            let allowed = std::iter::once("\"kind\"".to_owned())
                .chain(
                    variant
                        .fields
                        .iter()
                        .map(|field| format!("\"{}\"", snake_to_camel(field.name))),
                )
                .collect::<Vec<_>>()
                .join(", ");
            let unexpected_checks = format!(
                "      for (const key of Object.keys(value)) {{\n        if (![{allowed}].includes(key)) {{\n          throw new InvalidRequestError(\"{function}.\" + key, \"{name}.{} does not accept \" + key);\n        }}\n      }}\n",
                variant.name,
            );
            let fields = variant
                .fields
                .iter()
                .map(|field| {
                    format!(
                        ", {}: {}",
                        field.name,
                        to_expression(&field.ty, &format!("value.{}", snake_to_camel(field.name)),)
                    )
                })
                .collect::<String>();
            format!(
                "    case \"{}\": {{\n{unexpected_checks}      return {{ kind: \"{}\"{fields} }};\n    }}\n",
                variant.name, variant.name,
            )
        })
        .collect::<String>();
    format!(
        "export function {function}FromWire(value: Wire.{name}Wire): Model.{name} {{\n  switch (value.kind) {{\n{from_arms}  }}\n  return assertNever(value);\n}}\n\nexport function {function}ToWire(value: Model.{name}): Wire.{name}Wire {{\n  switch (value.kind) {{\n{to_arms}  }}\n  return assertNever(value);\n}}\n\n"
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

fn from_expression(schema: &Schema, ty: &Type, value: &str, field: &str) -> String {
    match ty {
        Type::String | Type::Boolean | Type::Number => value.to_owned(),
        Type::UnsignedInteger => format!("unsignedInteger({value}, \"{field}\")"),
        Type::Decimal => format!("Model.Decimal.parse({value})"),
        Type::Timestamp => format!("Model.Timestamp.fromNanoseconds(BigInt({value}))"),
        Type::Identifier(name) => match schema.identifier(name) {
            Some(identifier) if identifier.open => format!("Model.{name}.other({value})"),
            _ => format!("identifier(Model.{name}.values, {value}, \"{field}\")"),
        },
        Type::Named(name) if name.ends_with("Wire") => {
            format!(
                "{}FromWire({value})",
                lower_camel(name.trim_end_matches("Wire"))
            )
        }
        Type::Named(_) => value.to_owned(),
        Type::Optional(inner) => format!(
            "{value} === null ? null : {}",
            from_expression(schema, inner, value, field)
        ),
        Type::List(inner) => format!(
            "{value}.map((item) => {})",
            from_expression(schema, inner, "item", field)
        ),
        Type::Tuple(items) => format!(
            "[{}]",
            items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    from_expression(schema, item, &format!("{value}[{index}]"), field)
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn to_expression(ty: &Type, value: &str) -> String {
    match ty {
        Type::String | Type::Boolean | Type::Number => value.to_owned(),
        Type::UnsignedInteger => format!("{value}.toString()"),
        Type::Decimal => format!("{value}.toString()"),
        Type::Timestamp => format!("{value}.nanosecondsSinceEpoch.toString()"),
        Type::Identifier(_) => format!("{value}.id"),
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
    clientId: value.client_id,
  };
  const market = marketFromWire(value.market);
  const side = identifier(Model.Side.values, value.side, "side");
  if (value.order_type === Model.OrderType.Market.id) {
    return Model.OrderRequest.market(market, side, size, options);
  }
  if (value.order_type === Model.OrderType.Limit.id && value.price !== null) {
    return Model.OrderRequest.limit(market, side, size, Model.Decimal.parse(value.price), options);
  }
  if (value.order_type === Model.OrderType.Best.id && value.price === null && options.timeInForce !== null) {
    return Model.OrderRequest.best(market, side, size, options.timeInForce, options);
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
    reduce_only: value.reduceOnly, client_id: value.clientId,
  };
}

"#;

const STREAM_CONFIG_CODEC: &str = r#"export function streamConfigFromWire(value: Wire.StreamConfigWire): Model.StreamConfig {
  return new Model.StreamConfig({
    maxReconnectAttempts: value.max_reconnect_attempts,
    initialReconnectDelayMs: safeUnsignedInteger(value.initial_reconnect_delay_ms, "initial_reconnect_delay_ms"),
    maxReconnectDelayMs: safeUnsignedInteger(value.max_reconnect_delay_ms, "max_reconnect_delay_ms"),
    idleTimeoutMs: safeUnsignedInteger(value.idle_timeout_ms, "idle_timeout_ms"),
    bufferSize: safeUnsignedInteger(value.buffer_size, "buffer_size"),
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

const ORDER_HISTORY_REQUEST_CODEC: &str = r#"export function orderHistoryRequestFromWire(value: Wire.OrderHistoryRequestWire): Model.OrderHistoryRequest {
  return new Model.OrderHistoryRequest(
    value.market === null ? null : marketFromWire(value.market),
    value.statuses.map((status) => identifier(Model.OrderStatus.values, status, "statuses")),
    value.from === null ? null : Model.Timestamp.fromNanoseconds(BigInt(value.from)),
    value.to === null ? null : Model.Timestamp.fromNanoseconds(BigInt(value.to)),
    value.cursor === null ? null : new Model.Cursor(value.cursor),
    value.limit,
  );
}

export function orderHistoryRequestToWire(value: Model.OrderHistoryRequest): Wire.OrderHistoryRequestWire {
  return {
    market: value.market === null ? null : marketToWire(value.market),
    statuses: value.statuses.map((status) => status.id),
    from: value.from?.nanosecondsSinceEpoch.toString() ?? null,
    to: value.to?.nanosecondsSinceEpoch.toString() ?? null,
    cursor: value.cursor?.value ?? null,
    limit: value.limit,
  };
}

"#;

const TRANSFER_HISTORY_REQUEST_CODEC: &str = r#"export function transferHistoryRequestFromWire(value: Wire.TransferHistoryRequestWire): Model.TransferHistoryRequest {
  return new Model.TransferHistoryRequest(
    value.asset,
    value.network === null ? null : Model.Network.other(value.network),
    value.cursor === null ? null : new Model.Cursor(value.cursor),
    value.limit,
  );
}

export function transferHistoryRequestToWire(value: Model.TransferHistoryRequest): Wire.TransferHistoryRequestWire {
  return {
    asset: value.asset,
    network: value.network?.id ?? null,
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

export function hyperliquidMarketStreamItemFromWire(value: Wire.HyperliquidMarketStreamItemWire): StreamItem<Model.HyperliquidMarketEvent> {
  return value.kind === "error"
    ? new StreamError(errorFromWire(value.error))
    : new StreamEvent(hyperliquidMarketEventFromWire(value.event));
}

export function hyperliquidAccountStreamItemFromWire(value: Wire.HyperliquidAccountStreamItemWire): StreamItem<Model.HyperliquidAccountEvent> {
  return value.kind === "error"
    ? new StreamError(errorFromWire(value.error))
    : new StreamEvent(hyperliquidAccountEventFromWire(value.event));
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

#[cfg(test)]
mod tests {
    use maxt_bindings_common::schema::binding_schema;

    use super::render;

    #[test]
    fn unsigned_integers_preserve_typescript_bigints() {
        let output = render(&binding_schema());

        assert!(output.contains("function unsignedInteger(value: string, field: string): bigint"));
        assert!(output.contains(
            "minimum_deposit_confirmations: value.minimumDepositConfirmations.toString(),"
        ));
        assert!(!output.contains("Number(value);"));
        assert!(output.contains("for (const key of Object.keys(value))"));
        assert!(output.contains(
            "upbitBatchCancelScope.\" + key, \"UpbitBatchCancelScope.all does not accept \" + key"
        ));
        assert!(output.contains(
            "upbitBatchCancelRequest.\" + key, \"UpbitBatchCancelRequest does not accept \" + key"
        ));
    }
}
