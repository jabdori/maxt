use maxt_bindings_common::schema::{
    ApiType, Argument, ClientComposition, Operation, Provider, ProviderMethod, ProviderMethodKind,
    Schema,
};

use crate::typescript_contract::{HEADER, lower_camel};

pub(crate) fn render(schema: &Schema) -> String {
    let mut output = String::from(HEADER);
    output.push_str(&format!(
        r#"
import {{ AdapterError, MaxtError, UnsupportedError, errorFromWire, errorToWire }} from "../errors.js";
import * as Model from "../models.js";
import {{ ensureInitialized, getBackend }} from "../native.js";
import {{ AccountStream, MarketStream, StreamError }} from "../stream.js";
import * as Codec from "./codec.js";
import type * as Wire from "./contract.js";

export const NATIVE_API_VERSION = {} as const;

export type NativeOutcome<T> =
  | {{ readonly ok: true; readonly value: T }}
  | {{ readonly ok: false; readonly error: Wire.ErrorWire }};

export interface RawNativeClient {{
  exchange(): string;
  supports(feature: string): boolean;
"#,
        schema.native_api_version
    ));
    for operation in schema.adapter_operations {
        for method in operation.client_methods {
            output.push_str(&format!(
                "  {}({}): Promise<unknown>;\n",
                method.native_name,
                raw_parameters(method.arguments)
            ));
        }
    }
    for method in schema.client_compositions {
        output.push_str(&format!(
            "  {}({}): Promise<unknown>;\n",
            method.language_name,
            raw_parameters(method.arguments)
        ));
    }
    output.push_str(
        "  streamNext(id: string): Promise<unknown>;\n  streamClose(id: string): Promise<unknown>;\n}\n\n",
    );
    for provider in schema.providers {
        render_raw_provider(&mut output, provider);
    }
    output.push_str(
        r#"export interface RawForeignAdapterCallbacks {
  dispatch(call: string): Promise<string>;
  streamNext(id: string): Promise<string>;
  streamClose(id: string): Promise<string>;
}

export interface RawNativeModule {
  readonly NATIVE_API_VERSION: number;
  createCustomClient(
    exchange: string,
    features: string[],
    dispatch: RawForeignAdapterCallbacks["dispatch"],
    streamNext: RawForeignAdapterCallbacks["streamNext"],
    streamClose: RawForeignAdapterCallbacks["streamClose"],
  ): RawNativeClient;
"#,
    );
    for provider in schema.providers {
        output.push_str(&format!(
            "  {}(options: string): {};\n",
            provider.native_factory,
            raw_handle_name(provider)
        ));
    }
    output.push_str("}\n\n");

    output.push_str(
        r#"export interface NativeStreamHandle<I> {
  next(): Promise<NativeOutcome<I | null>>;
  close(): Promise<NativeOutcome<null>>;
}

export interface NativeClientHandle {
  raw(): RawNativeClient;
  exchange(): string;
  supports(feature: string): boolean;
"#,
    );
    for operation in schema.adapter_operations {
        for method in operation.client_methods {
            output.push_str(&format!(
                "  {}({}): Promise<NativeOutcome<{}>>;\n",
                method.native_name,
                wire_parameters(method.arguments, schema),
                native_result_type(operation.result, schema)
            ));
        }
    }
    for method in schema.client_compositions {
        output.push_str(&format!(
            "  {}({}): Promise<NativeOutcome<{}>>;\n",
            method.language_name,
            wire_parameters(method.arguments, schema),
            native_result_type(method.result, schema)
        ));
    }
    output.push_str("}\n\n");
    output.push_str(
        r#"export interface ForeignAdapterCallbacks {
  dispatch(call: Wire.AdapterCallWire): Promise<NativeOutcome<Wire.AdapterReplyWire>>;
  streamNext(id: string): Promise<NativeOutcome<Wire.MarketStreamItemWire | Wire.AccountStreamItemWire | null>>;
  streamClose(id: string): Promise<NativeOutcome<null>>;
}

"#,
    );
    for provider in schema.providers {
        render_typed_provider(&mut output, provider, schema);
    }
    output.push_str(
        r#"export interface NativeBackend {
  initialize(options: { readonly wasmUrl: string | null; readonly allowInsecureBrowserCredentials: boolean; readonly relayUrl: string | null }): Promise<void>;
  customClient(exchange: string, features: readonly string[], callbacks: ForeignAdapterCallbacks): NativeClientHandle;
"#,
    );
    for provider in schema.providers {
        output.push_str(&format!(
            "  {}(options: Wire.{}): {};\n",
            provider.exchange, provider.options_wire, provider.native_handle
        ));
    }
    output.push_str("}\n\n");
    output.push_str(JSON_BACKEND_HELPERS);
    render_json_backend(&mut output, schema);
    output.push_str(ADAPTER_STREAMS);
    render_adapter(&mut output, schema);
    output.push_str(CUSTOM_CALLBACKS_PREFIX);
    render_dispatch(&mut output, schema);
    output.push_str(CUSTOM_CALLBACKS_SUFFIX);
    render_client(&mut output, schema);
    render_builtin_base(&mut output, schema);
    for provider in schema.providers {
        render_provider_class(&mut output, provider, schema);
    }
    output.truncate(output.trim_end().len());
    output.push('\n');
    output
}

fn raw_handle_name(provider: &Provider) -> String {
    format!("Raw{}", provider.native_handle)
}

fn raw_parameters(arguments: &[Argument]) -> String {
    arguments
        .iter()
        .map(|argument| {
            let ty = if matches!(argument.ty, ApiType::Client) {
                "RawNativeClient"
            } else {
                "string"
            };
            format!("{}: {ty}", argument.name)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn public_type(ty: ApiType) -> String {
    match ty {
        ApiType::Client => "Client<Adapter>".to_owned(),
        ApiType::String => "string".to_owned(),
        ApiType::Boolean => "boolean".to_owned(),
        ApiType::Number => "number".to_owned(),
        ApiType::Named(name) => format!("Model.{name}"),
        ApiType::OptionalString => "string | null".to_owned(),
        ApiType::OptionalNumber => "number | null".to_owned(),
        ApiType::OptionalNamed(name) => format!("Model.{name} | null"),
        ApiType::List(name) => format!("readonly {}[]", named_public_type(name)),
        ApiType::PairList(left, right) => format!(
            "readonly (readonly [{}, {}])[]",
            named_public_type(left),
            named_public_type(right)
        ),
        ApiType::Page(name) => format!("Model.Page<Model.{name}>"),
        ApiType::Handle(name) | ApiType::HandleToken(name) => name.to_owned(),
        ApiType::MarketStream => "MarketStream".to_owned(),
        ApiType::AccountStream => "AccountStream".to_owned(),
        ApiType::Unit => "void".to_owned(),
    }
}

fn named_public_type(name: &str) -> String {
    match name {
        "String" => "string".to_owned(),
        _ => format!("Model.{name}"),
    }
}

fn wire_named_type(name: &str, schema: &Schema) -> String {
    if matches!(name, "String") {
        return "string".to_owned();
    }
    if matches!(name, "Timestamp") {
        return "Wire.TimestampWire".to_owned();
    }
    if matches!(name, "Cursor") {
        return "string".to_owned();
    }
    if matches!(name, "Decimal") {
        return "string".to_owned();
    }
    if schema.has_identifier(name) {
        return "string".to_owned();
    }
    format!("Wire.{name}Wire")
}

fn wire_type(ty: ApiType, schema: &Schema) -> String {
    match ty {
        ApiType::Client => "NativeClientHandle".to_owned(),
        ApiType::String => "string".to_owned(),
        ApiType::Boolean => "boolean".to_owned(),
        ApiType::Number => "number".to_owned(),
        ApiType::Named(name) => wire_named_type(name, schema),
        ApiType::OptionalString => "string | null".to_owned(),
        ApiType::OptionalNumber => "number | null".to_owned(),
        ApiType::OptionalNamed(name) => format!("{} | null", wire_named_type(name, schema)),
        ApiType::List(name) => format!("readonly {}[]", wire_named_type(name, schema)),
        ApiType::PairList(left, right) => format!(
            "readonly (readonly [{}, {}])[]",
            wire_named_type(left, schema),
            wire_named_type(right, schema)
        ),
        ApiType::Page(name) => format!("Wire.PageWire<{}>", wire_named_type(name, schema)),
        ApiType::Handle(name) => format!("Wire.{name}Wire"),
        ApiType::HandleToken(_) => "string".to_owned(),
        ApiType::MarketStream => "NativeStreamHandle<Wire.MarketStreamItemWire>".to_owned(),
        ApiType::AccountStream => "NativeStreamHandle<Wire.AccountStreamItemWire>".to_owned(),
        ApiType::Unit => "null".to_owned(),
    }
}

fn native_result_type(ty: ApiType, schema: &Schema) -> String {
    wire_type(ty, schema)
}

fn public_parameters(arguments: &[Argument]) -> String {
    arguments
        .iter()
        .map(|argument| {
            let default = argument
                .default
                .map(|value| format!(" = {value}"))
                .unwrap_or_default();
            format!("{}: {}{}", argument.name, public_type(argument.ty), default)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn wire_parameters(arguments: &[Argument], schema: &Schema) -> String {
    arguments
        .iter()
        .map(|argument| format!("{}: {}", argument.name, wire_type(argument.ty, schema)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_raw_provider(output: &mut String, provider: &Provider) {
    output.push_str(&format!(
        "export interface {} {{\n  client(): RawNativeClient;\n",
        raw_handle_name(provider)
    ));
    for method in provider.methods {
        match method.kind {
            ProviderMethodKind::Property => output.push_str(&format!(
                "  {}(): {};\n",
                method.name,
                raw_sync_type(method.result)
            )),
            ProviderMethodKind::Async => output.push_str(&format!(
                "  {}({}): Promise<unknown>;\n",
                method.name,
                raw_parameters(method.arguments)
            )),
        }
    }
    output.push_str("}\n\n");
}

fn raw_sync_type(ty: ApiType) -> &'static str {
    match ty {
        ApiType::Boolean => "boolean",
        _ => "string",
    }
}

fn render_typed_provider(output: &mut String, provider: &Provider, schema: &Schema) {
    output.push_str(&format!(
        "export interface {} {{\n  client(): NativeClientHandle;\n",
        provider.native_handle
    ));
    for method in provider.methods {
        match method.kind {
            ProviderMethodKind::Property => output.push_str(&format!(
                "  {}(): {};\n",
                method.name,
                wire_type(method.result, schema)
            )),
            ProviderMethodKind::Async => output.push_str(&format!(
                "  {}({}): Promise<NativeOutcome<{}>>;\n",
                method.name,
                wire_parameters(method.arguments, schema),
                native_result_type(method.result, schema)
            )),
        }
    }
    output.push_str("}\n\n");
}

fn render_json_backend(output: &mut String, schema: &Schema) {
    output.push_str("export function createJsonBackend(raw: RawNativeModule): NativeBackend {\n  return {\n    async initialize() {},\n    customClient(exchange, features, callbacks) {\n      const bridge: RawForeignAdapterCallbacks = {\n        dispatch: async (call) => Codec.stringifyWire(await callbacks.dispatch(JSON.parse(call) as Wire.AdapterCallWire)),\n        streamNext: async (id) => Codec.stringifyWire(await callbacks.streamNext(id)),\n        streamClose: async (id) => Codec.stringifyWire(await callbacks.streamClose(id)),\n      };\n      return wrapJsonClient(callFactory(() => raw.createCustomClient(\n        exchange, [...features], bridge.dispatch, bridge.streamNext, bridge.streamClose,\n      )));\n    },\n");
    for provider in schema.providers {
        output.push_str(&format!(
            "    {}(options) {{\n      const handle = callFactory(() => raw.{}(Codec.stringifyWire(options)));\n      return {{\n        client: () => wrapJsonClient(handle.client()),\n",
            provider.exchange, provider.native_factory
        ));
        for method in provider.methods {
            match method.kind {
                ProviderMethodKind::Property => output.push_str(&format!(
                    "        {}: () => handle.{}(),\n",
                    method.name, method.name
                )),
                ProviderMethodKind::Async => output.push_str(&format!(
                    "        {}: ({}) => handle.{}({}) as Promise<NativeOutcome<{}>>,\n",
                    method.name,
                    wire_parameters(method.arguments, schema),
                    method.name,
                    json_arguments(method.arguments),
                    native_result_type(method.result, schema)
                )),
            }
        }
        output.push_str("      };\n    },\n");
    }
    output.push_str("  };\n}\n\n");
    output.push_str("function wrapJsonClient(raw: RawNativeClient): NativeClientHandle {\n  const stream = <I>(reference: Wire.NativeStreamReferenceWire): NativeStreamHandle<I> => ({\n    next: () => raw.streamNext(Codec.stringifyWire(reference.id)) as Promise<NativeOutcome<I | null>>,\n    close: () => raw.streamClose(Codec.stringifyWire(reference.id)) as Promise<NativeOutcome<null>>,\n  });\n  return {\n    raw: () => raw,\n    exchange: () => raw.exchange(),\n    supports: (feature) => raw.supports(feature),\n");
    for operation in schema.adapter_operations {
        for method in operation.client_methods {
            let result = native_result_type(operation.result, schema);
            if matches!(
                operation.result,
                ApiType::MarketStream | ApiType::AccountStream
            ) {
                let item = if matches!(operation.result, ApiType::MarketStream) {
                    "Wire.MarketStreamItemWire"
                } else {
                    "Wire.AccountStreamItemWire"
                };
                output.push_str(&format!(
                    "    async {}({}) {{\n      const outcome = await raw.{}({}) as NativeOutcome<Wire.NativeStreamReferenceWire>;\n      return outcome.ok ? {{ ok: true, value: stream<{}>(outcome.value) }} : outcome;\n    }},\n",
                    method.native_name,
                    wire_parameters(method.arguments, schema),
                    method.native_name,
                    json_arguments(method.arguments),
                    item
                ));
            } else {
                output.push_str(&format!(
                    "    {}: ({}) => raw.{}({}) as Promise<NativeOutcome<{}>>,\n",
                    method.native_name,
                    wire_parameters(method.arguments, schema),
                    method.native_name,
                    json_arguments(method.arguments),
                    result
                ));
            }
        }
    }
    for method in schema.client_compositions {
        output.push_str(&format!(
            "    {}: ({}) => raw.{}({}) as Promise<NativeOutcome<{}>>,\n",
            method.language_name,
            wire_parameters(method.arguments, schema),
            method.language_name,
            json_arguments(method.arguments),
            native_result_type(method.result, schema)
        ));
    }
    output.push_str("  };\n}\n\n");
}

fn json_arguments(arguments: &[Argument]) -> String {
    arguments
        .iter()
        .map(|argument| {
            if matches!(argument.ty, ApiType::Client) {
                format!("{}.raw()", argument.name)
            } else {
                format!("Codec.stringifyWire({})", argument.name)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_adapter(output: &mut String, schema: &Schema) {
    output.push_str("export abstract class Adapter {\n  abstract get exchange(): Model.Exchange;\n  abstract get features(): ReadonlySet<Model.Feature>;\n\n  supports(feature: Model.Feature): boolean { return this.features.has(feature); }\n\n");
    for operation in schema.adapter_operations {
        output.push_str(&format!(
            "  {}({}): Promise<{}> {{\n    return Promise.reject(new UnsupportedError(\n      featureById(\"{}\"), this.exchange, \"feature is not supported\",\n    ));\n  }}\n\n",
            operation.language_name,
            public_parameters(operation.arguments),
            public_type(operation.result),
            operation.feature
        ));
    }
    output.push_str("}\n\n");
}

fn render_dispatch(output: &mut String, schema: &Schema) {
    for operation in schema.adapter_operations {
        output.push_str(&dispatch_arm(operation));
    }
}

fn dispatch_arm(operation: &Operation) -> String {
    match operation.rust_name {
        "markets" => "        case \"markets\": return ok({ kind: \"markets\", value: (await this.adapter.markets(call.market_kind === \"spot\" ? Model.MarketKind.Spot : Model.MarketKind.Perpetual)).map(Codec.marketInfoToWire) });\n".to_owned(),
        "trades" => "        case \"trades\": return ok({ kind: \"trades\", value: (await this.adapter.trades(Codec.marketFromWire(call.market), call.limit)).map(Codec.tradeToWire) });\n".to_owned(),
        "order_book" => "        case \"order_book\": return ok({ kind: \"order_book\", value: Codec.orderBookToWire(await this.adapter.orderBook(Codec.marketFromWire(call.market), call.depth)) });\n".to_owned(),
        "ticker" => "        case \"ticker\": return ok({ kind: \"ticker\", value: Codec.tickerToWire(await this.adapter.ticker(Codec.marketFromWire(call.market))) });\n".to_owned(),
        "candles" => "        case \"candles\": return ok({ kind: \"candles\", value: (await this.adapter.candles(Codec.candleRequestFromWire(call.request))).map(Codec.candleToWire) });\n".to_owned(),
        "subscribe" => "        case \"subscribe\": { this.#streams.begin(call.stream_id); try { const value = await this.adapter.subscribe(Codec.subscriptionFromWire(call.subscription), Codec.streamConfigFromWire(call.config)); await this.#streams.register(call.stream_id, value); return ok({ kind: \"market_stream\", stream_id: call.stream_id }); } catch (error) { this.#streams.abort(call.stream_id); throw error; } }\n".to_owned(),
        "balances" => "        case \"balances\": return ok({ kind: \"balances\", value: (await this.adapter.balances()).map(Codec.balanceToWire) });\n".to_owned(),
        "order_rules" => "        case \"order_rules\": return ok({ kind: \"order_rules\", value: Codec.orderRulesToWire(await this.adapter.orderRules(Codec.marketFromWire(call.market))) });\n".to_owned(),
        "asset_networks" => "        case \"asset_networks\": return ok({ kind: \"asset_networks\", value: (await this.adapter.assetNetworks(call.asset)).map(Codec.assetNetworkToWire) });\n".to_owned(),
        "deposit_addresses" => "        case \"deposit_addresses\": return ok({ kind: \"deposit_addresses\", value: (await this.adapter.depositAddresses()).map(Codec.depositAddressEntryToWire) });\n".to_owned(),
        "deposit_address" => "        case \"deposit_address\": return ok({ kind: \"deposit_address\", value: Codec.depositAddressToWire(await this.adapter.depositAddress(Codec.depositAddressRequestFromWire(call.request))) });\n".to_owned(),
        "create_deposit_address" => "        case \"create_deposit_address\": return ok({ kind: \"create_deposit_address\", value: Codec.depositAddressToWire(await this.adapter.createDepositAddress(Codec.depositAddressRequestFromWire(call.request))) });\n".to_owned(),
        "prepare_withdrawal" => "        case \"prepare_withdrawal\": return ok({ kind: \"withdrawal_quote\", value: Codec.withdrawalQuoteToWire(await this.adapter.prepareWithdrawal(Codec.withdrawRequestFromWire(call.request))) });\n".to_owned(),
        "withdraw" => "        case \"withdraw\": return ok({ kind: \"withdrawal\", value: Codec.withdrawalToWire(await this.adapter.withdraw(Codec.withdrawRequestFromWire(call.request))) });\n".to_owned(),
        "deposit" => "        case \"deposit\": return ok({ kind: \"deposit\", value: Codec.depositToWire(await this.adapter.deposit(Codec.transferLookupRequestFromWire(call.request))) });\n".to_owned(),
        "withdrawal" => "        case \"withdrawal\": return ok({ kind: \"withdrawal_lookup\", value: Codec.withdrawalToWire(await this.adapter.withdrawal(Codec.transferLookupRequestFromWire(call.request))) });\n".to_owned(),
        "cancel_withdrawal" => "        case \"cancel_withdrawal\": await this.adapter.cancelWithdrawal(call.withdrawal_id); return ok({ kind: \"unit\" });\n".to_owned(),
        "deposits" => "        case \"deposits\": { const value = await this.adapter.deposits(Codec.transferHistoryRequestFromWire(call.request)); return ok({ kind: \"deposits\", value: { items: value.items.map(Codec.depositToWire), next: value.next?.value ?? null } }); }\n".to_owned(),
        "withdrawals" => "        case \"withdrawals\": { const value = await this.adapter.withdrawals(Codec.transferHistoryRequestFromWire(call.request)); return ok({ kind: \"withdrawals\", value: { items: value.items.map(Codec.withdrawalToWire), next: value.next?.value ?? null } }); }\n".to_owned(),
        "open_orders" => "        case \"open_orders\": return ok({ kind: \"open_orders\", value: (await this.adapter.openOrders(call.market === null ? null : Codec.marketFromWire(call.market))).map(Codec.orderToWire) });\n".to_owned(),
        "order" => "        case \"order\": return ok({ kind: \"order\", value: Codec.orderToWire(await this.adapter.order(Codec.marketFromWire(call.market), call.order_id)) });\n".to_owned(),
        "order_by_client_id" => "        case \"order_by_client_id\": return ok({ kind: \"order\", value: Codec.orderToWire(await this.adapter.orderByClientId(Codec.marketFromWire(call.market), call.client_id)) });\n".to_owned(),
        "orders_by_ids" => "        case \"orders_by_ids\": return ok({ kind: \"orders_by_ids\", value: (await this.adapter.ordersByIds(Codec.orderLookupRequestFromWire(call.request))).map(Codec.orderToWire) });\n".to_owned(),
        "order_history" => "        case \"order_history\": { const value = await this.adapter.orderHistory(Codec.orderHistoryRequestFromWire(call.request)); return ok({ kind: \"order_history\", value: { items: value.items.map(Codec.orderToWire), next: value.next?.value ?? null } }); }\n".to_owned(),
        "subscribe_account" => "        case \"subscribe_account\": { this.#streams.begin(call.stream_id); try { const value = await this.adapter.subscribeAccount(Codec.streamConfigFromWire(call.config)); await this.#streams.register(call.stream_id, value); return ok({ kind: \"account_stream\", stream_id: call.stream_id }); } catch (error) { this.#streams.abort(call.stream_id); throw error; } }\n".to_owned(),
        "place_order" => "        case \"place_order\": return ok({ kind: \"place_order\", value: Codec.orderToWire(await this.adapter.placeOrder(Codec.orderRequestFromWire(call.request))) });\n".to_owned(),
        "cancel_order" => "        case \"cancel_order\": await this.adapter.cancelOrder(Codec.marketFromWire(call.market), call.order_id); return ok({ kind: \"unit\" });\n".to_owned(),
        "cancel_order_by_client_id" => "        case \"cancel_order_by_client_id\": await this.adapter.cancelOrderByClientId(Codec.marketFromWire(call.market), call.client_id); return ok({ kind: \"unit\" });\n".to_owned(),
        "cancel_orders" => "        case \"cancel_orders\": return ok({ kind: \"cancel_orders\", value: Codec.cancelOrdersResultToWire(await this.adapter.cancelOrders(Codec.cancelOrdersRequestFromWire(call.request))) });\n".to_owned(),
        "positions" => "        case \"positions\": return ok({ kind: \"positions\", value: (await this.adapter.positions(call.market === null ? null : Codec.marketFromWire(call.market))).map(Codec.positionToWire) });\n".to_owned(),
        "margin_summary" => "        case \"margin_summary\": return ok({ kind: \"margin_summary\", value: Codec.marginSummaryToWire(await this.adapter.marginSummary()) });\n".to_owned(),
        "funding_rates" => "        case \"funding_rates\": { const value = await this.adapter.fundingRates(Codec.historyRequestFromWire(call.request)); return ok({ kind: \"funding_rates\", value: { items: value.items.map(Codec.fundingRateToWire), next: value.next?.value ?? null } }); }\n".to_owned(),
        "funding_payments" => "        case \"funding_payments\": { const value = await this.adapter.fundingPayments(Codec.historyRequestFromWire(call.request)); return ok({ kind: \"funding_payments\", value: { items: value.items.map(Codec.fundingPaymentToWire), next: value.next?.value ?? null } }); }\n".to_owned(),
        "set_margin" => "        case \"set_margin\": await this.adapter.setMargin(Codec.marginRequestFromWire(call.request)); return ok({ kind: \"unit\" });\n".to_owned(),
        name => panic!("TypeScript adapter dispatcher is missing {name}"),
    }
}

fn render_client(output: &mut String, schema: &Schema) {
    output.push_str("export class Client<A extends Adapter> {\n  readonly #native: NativeClientHandle;\n  constructor(readonly adapter: A) {\n    const bound = nativeClients.get(adapter);\n    this.#native = bound ?? getBackend().customClient(\n      adapter.exchange.id, [...adapter.features].map((feature) => feature.id), new CustomCallbacks(adapter),\n    );\n    if (bound === undefined) nativeClients.set(adapter, this.#native);\n  }\n\n  get exchange(): Model.Exchange { return this.adapter.exchange; }\n  supports(feature: Model.Feature): boolean { return this.#native.supports(feature.id); }\n\n");
    for operation in schema.adapter_operations {
        for method in operation.client_methods {
            output.push_str(&render_client_method(
                operation,
                method.name,
                method.native_name,
                method.arguments,
                schema,
            ));
        }
    }
    for method in schema.client_compositions {
        output.push_str(&render_composed_client_method(method, schema));
    }
    output.push_str("}\n\n");
}

fn render_composed_client_method(method: &ClientComposition, schema: &Schema) -> String {
    let encoded = method
        .arguments
        .iter()
        .map(|argument| encode_argument(argument, schema))
        .collect::<Vec<_>>()
        .join(", ");
    let call = format!("this.#native.{}({encoded})", method.language_name);
    let ApiType::Named(model) = method.result else {
        panic!("composed Client methods currently return one model")
    };
    format!(
        "  async {}({}): Promise<{}> {{\n    await ensureInitialized();\n    return Codec.{}FromWire(Codec.unwrapOutcome(await {call}));\n  }}\n\n",
        method.language_name,
        public_parameters(method.arguments),
        public_type(method.result),
        lower_camel(model),
    )
}

fn render_client_method(
    operation: &Operation,
    name: &str,
    native_name: &str,
    arguments: &[Argument],
    schema: &Schema,
) -> String {
    let encoded = arguments
        .iter()
        .map(|argument| encode_argument(argument, schema))
        .collect::<Vec<_>>()
        .join(", ");
    let call = format!("this.#native.{native_name}({encoded})");
    let body = match operation.result {
        ApiType::Named(model) => format!(
            "return Codec.{}FromWire(Codec.unwrapOutcome(await {call}));",
            lower_camel(model)
        ),
        ApiType::List(model) => format!(
            "return Codec.unwrapOutcome(await {call}).map(Codec.{}FromWire);",
            lower_camel(model)
        ),
        ApiType::Page(model) => format!(
            "return Codec.pageFromWire(Codec.unwrapOutcome(await {call}), Codec.{}FromWire);",
            lower_camel(model)
        ),
        ApiType::MarketStream => format!(
            "const handle = Codec.unwrapOutcome(await {call}); return new MarketStream(nativeItems(handle, Codec.marketStreamItemFromWire), async () => {{ Codec.unwrapOutcome(await handle.close()); }});"
        ),
        ApiType::AccountStream => format!(
            "const handle = Codec.unwrapOutcome(await {call}); return new AccountStream(nativeItems(handle, Codec.accountStreamItemFromWire), async () => {{ Codec.unwrapOutcome(await handle.close()); }});"
        ),
        ApiType::Unit => format!("Codec.unwrapOutcome(await {call});"),
        _ => panic!("unsupported client result for {}", operation.rust_name),
    };
    format!(
        "  async {name}({}): Promise<{}> {{\n    await ensureInitialized();\n    {body}\n  }}\n\n",
        public_parameters(arguments),
        public_type(operation.result)
    )
}

fn encode_argument(argument: &Argument, schema: &Schema) -> String {
    let name = argument.name;
    match argument.ty {
        ApiType::Client => format!("{name}.#native"),
        ApiType::Number => format!("Codec.checkedU32({name}, \"{name}\")"),
        ApiType::OptionalNumber => format!("Codec.checkedOptionalU32({name}, \"{name}\")"),
        ApiType::Named(value) if schema.has_identifier(value) => format!("{name}.id"),
        ApiType::Named("Timestamp") => format!("{name}.nanosecondsSinceEpoch.toString()"),
        ApiType::Named("Cursor") => format!("{name}.value"),
        ApiType::Named("Decimal") => format!("{name}.toString()"),
        ApiType::Named("BinanceListenKey") => format!("{name}.id"),
        ApiType::HandleToken(_) => format!("{name}.id"),
        ApiType::Named(value) => format!("Codec.{}ToWire({name})", lower_camel(value)),
        ApiType::OptionalNamed("Timestamp") => {
            format!("{name}?.nanosecondsSinceEpoch.toString() ?? null")
        }
        ApiType::OptionalNamed("Cursor") => format!("{name}?.value ?? null"),
        ApiType::OptionalNamed(value) => format!(
            "{name} === null ? null : Codec.{}ToWire({name})",
            lower_camel(value)
        ),
        ApiType::List("String") => name.to_owned(),
        ApiType::List(value) => format!("{name}.map(Codec.{}ToWire)", lower_camel(value)),
        _ => name.to_owned(),
    }
}

fn render_provider_class(output: &mut String, provider: &Provider, schema: &Schema) {
    output.push_str(&provider_constructor(provider));
    for method in provider.methods {
        output.push_str(&render_provider_method(provider, method, schema));
    }
    output.push_str("}\n\n");
}

fn render_builtin_base(output: &mut String, schema: &Schema) {
    output.push_str(BUILTIN_BASE_PREFIX);
    for operation in schema.adapter_operations {
        let arguments = public_parameters(operation.arguments);
        let names = operation
            .arguments
            .iter()
            .map(|argument| argument.name)
            .collect::<Vec<_>>()
            .join(", ");
        let body = match operation.rust_name {
            "open_orders" => {
                "return market === null ? new Client(this).openOrders() : new Client(this).openOrdersOn(market);".to_owned()
            }
            "positions" => {
                "return market === null ? new Client(this).positions() : new Client(this).positionsOn(market);".to_owned()
            }
            _ => {
                let method = operation
                    .client_methods
                    .iter()
                    .find(|method| method.arguments.len() == operation.arguments.len())
                    .unwrap_or_else(|| panic!("no Client mapping for {}", operation.rust_name));
                format!("return new Client(this).{}({names});", method.name)
            }
        };
        output.push_str(&format!(
            "  {}({}): Promise<{}> {{ {} }}\n",
            operation.language_name,
            arguments,
            public_type(operation.result),
            body
        ));
    }
    output.push_str(BUILTIN_BASE_SUFFIX);
}

fn provider_constructor(provider: &Provider) -> String {
    match provider.exchange {
        "upbit" => r#"export class UpbitAdapter extends NativeAdapter {
  readonly #provider: NativeUpbitHandle;
  constructor(options: { readonly region?: Model.UpbitRegion; readonly accessKey?: string; readonly secretKey?: string } = {}) {
    const provider = getBackend().upbit({ region: (options.region ?? Model.UpbitRegion.Korea).id, access_key: options.accessKey ?? null, secret_key: options.secretKey ?? null });
    super(provider.client()); this.#provider = provider;
  }
  static withRegion(region: Model.UpbitRegion, options: { readonly accessKey?: string; readonly secretKey?: string } = {}): UpbitAdapter {
    return new UpbitAdapter({ region, ...options });
  }
"#.to_owned(),
        "bithumb" => r#"export class BithumbAdapter extends NativeAdapter {
  readonly #provider: NativeBithumbHandle;
  constructor(options: { readonly accessKey?: string; readonly secretKey?: string } = {}) {
    const provider = getBackend().bithumb({ access_key: options.accessKey ?? null, secret_key: options.secretKey ?? null });
    super(provider.client()); this.#provider = provider;
  }
"#.to_owned(),
        "binance" => r#"export class BinanceAdapter extends NativeAdapter {
  readonly #provider: NativeBinanceHandle;
  private constructor(venue: Model.BinanceMarket, options: { readonly apiKey?: string; readonly secretKey?: string }) {
    const provider = getBackend().binance({ venue: venue.id, api_key: options.apiKey ?? null, secret_key: options.secretKey ?? null });
    super(provider.client()); this.#provider = provider;
  }
  static spot(options: { readonly apiKey?: string; readonly secretKey?: string } = {}): BinanceAdapter {
    return new BinanceAdapter(Model.BinanceMarket.Spot, options);
  }
  static usdMFutures(options: { readonly apiKey?: string; readonly secretKey?: string } = {}): BinanceAdapter {
    return new BinanceAdapter(Model.BinanceMarket.UsdMFutures, options);
  }
"#.to_owned(),
        "hyperliquid" => r#"export class HyperliquidAdapter extends NativeAdapter {
  readonly #provider: NativeHyperliquidHandle;
  constructor(options: { readonly address?: string; readonly privateKey?: string; readonly testnet?: boolean } = {}) {
    const provider = getBackend().hyperliquid({ testnet: options.testnet ?? false, address: options.address ?? null, private_key: options.privateKey ?? null });
    super(provider.client()); this.#provider = provider;
  }
  static testnet(options: { readonly address?: string; readonly privateKey?: string } = {}): HyperliquidAdapter {
    return new HyperliquidAdapter({ ...options, testnet: true });
  }
"#.to_owned(),
        exchange => panic!("TypeScript provider constructor is missing {exchange}"),
    }
}

fn render_provider_method(provider: &Provider, method: &ProviderMethod, schema: &Schema) -> String {
    if method.kind == ProviderMethodKind::Property {
        let expression = match method.result {
            ApiType::Boolean => format!("this.#provider.{}()", method.name),
            ApiType::Named(model) => format!(
                "Codec.{}FromWire(this.#provider.{}())",
                lower_camel(model),
                method.name
            ),
            _ => panic!("unsupported provider property {}", method.name),
        };
        return format!(
            "  get {}(): {} {{ return {}; }}\n",
            method.name,
            public_type(method.result),
            expression
        );
    }
    let encoded = method
        .arguments
        .iter()
        .map(|argument| encode_argument(argument, schema))
        .collect::<Vec<_>>()
        .join(", ");
    let call = format!("this.#provider.{}({encoded})", method.name);
    let body = match method.result {
        ApiType::Named(model) => format!(
            "return Codec.{}FromWire(Codec.unwrapOutcome(await {call}));",
            lower_camel(model)
        ),
        ApiType::List(model) => format!(
            "return Codec.unwrapOutcome(await {call}).map(Codec.{}FromWire);",
            lower_camel(model)
        ),
        ApiType::PairList(left, right) => format!(
            "return Codec.unwrapOutcome(await {call}).map(Codec.{}PairFromWire);",
            pair_decoder_name(left, right)
        ),
        ApiType::Page(model) => format!(
            "return Codec.pageFromWire(Codec.unwrapOutcome(await {call}), Codec.{}FromWire);",
            lower_camel(model)
        ),
        ApiType::String => format!("return Codec.unwrapOutcome(await {call});"),
        ApiType::Unit => format!("Codec.unwrapOutcome(await {call});"),
        ApiType::Handle(model) => format!(
            "const value = Codec.unwrapOutcome(await {call}); return new {model}(value.id, value.value);"
        ),
        _ => panic!(
            "unsupported provider result {}.{}",
            provider.exchange, method.name
        ),
    };
    let documentation = if provider.exchange == "upbit" && method.rust_name == "test_order" {
        "  /** Validates an Upbit order without creating it. The returned dry-run ID cannot be queried or cancelled, and its status is not a live order. */\n"
    } else {
        ""
    };
    format!(
        "{documentation}  async {}({}): Promise<{}> {{ await ensureInitialized(); {} }}\n",
        method.name,
        public_parameters(method.arguments),
        public_type(method.result),
        body
    )
}

fn pair_decoder_name(left: &str, right: &str) -> &'static str {
    match (left, right) {
        ("Market", "UpbitMarketEvent") => "upbitMarketEvent",
        ("Market", "String") => "marketString",
        ("Market", "BithumbMarketAlert") => "bithumbMarketAlert",
        _ => panic!("missing pair decoder for {left}, {right}"),
    }
}

const JSON_BACKEND_HELPERS: &str = r#"function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isWireError(value: unknown): value is Wire.ErrorWire {
  if (!isObject(value) || typeof value.kind !== "string") return false;
  switch (value.kind) {
    case "invalid_request": return typeof value.field === "string" && typeof value.detail === "string";
    case "transfer": return typeof value.transfer_kind === "string" && typeof value.detail === "string";
    case "unsupported": return typeof value.feature === "string" && typeof value.exchange === "string" && typeof value.detail === "string";
    case "adapter": case "auth": case "transport": case "decode": return typeof value.detail === "string";
    case "exchange": return typeof value.exchange === "string" && typeof value.code === "string" && typeof value.message === "string" && (value.status === null || typeof value.status === "number") && typeof value.exchange_kind === "string";
    default: return false;
  }
}

function callFactory<T>(factory: () => T): T {
  try { return factory(); } catch (cause) {
    const message = cause instanceof Error ? cause.message : null;
    if (message !== null) {
      try {
        const parsed: unknown = JSON.parse(message);
        if (isObject(parsed) && parsed.ok === false && isWireError(parsed.error)) {
          throw errorFromWire(parsed.error);
        }
      } catch (error) {
        if (error instanceof MaxtError) throw error;
      }
    }
    throw new AdapterError("native adapter factory failed", { cause });
  }
}

"#;

const ADAPTER_STREAMS: &str = r#"function ok<T>(value: T): NativeOutcome<T> { return { ok: true, value }; }
function failed(error: unknown): NativeOutcome<never> { return { ok: false, error: errorToWire(error) }; }
function featureById(id: string): Model.Feature {
  const feature = Model.Feature.values.find((value) => value.id === id);
  if (feature === undefined) throw new AdapterError(`unknown generated feature ${id}`);
  return feature;
}

class AdapterStreams {
  readonly #streams = new Map<string, MarketStream | AccountStream>();
  readonly #pending = new Set<string>();
  readonly #cancelled = new Set<string>();
  begin(id: string): void {
    if (this.#pending.has(id) || this.#streams.has(id)) {
      throw new AdapterError(`duplicate adapter stream \`${id}\``);
    }
    this.#pending.add(id);
  }
  async register(id: string, stream: MarketStream | AccountStream): Promise<void> {
    if (!this.#pending.delete(id)) {
      await stream.close();
      throw new AdapterError(`adapter stream \`${id}\` was not pending`);
    }
    if (this.#cancelled.delete(id)) { await stream.close(); return; }
    this.#streams.set(id, stream);
  }
  abort(id: string): void {
    this.#pending.delete(id);
    this.#cancelled.delete(id);
  }
  async next(id: string): Promise<NativeOutcome<Wire.MarketStreamItemWire | Wire.AccountStreamItemWire | null>> {
    const stream = this.#streams.get(id);
    if (stream === undefined) return failed(new AdapterError(`unknown adapter stream \`${id}\``));
    try {
      const item = await stream.next();
      if (item.done) { this.#streams.delete(id); await stream.close(); return ok(null); }
      if (item.value instanceof StreamError) return ok({ kind: "error", error: errorToWire(item.value.error) });
      return ok(stream instanceof MarketStream
        ? { kind: "event", event: Codec.marketEventToWire(item.value.event as Model.MarketEvent) }
        : { kind: "event", event: Codec.accountEventToWire(item.value.event as Model.AccountEvent) });
    } catch (error) { return failed(error); }
  }
  async close(id: string): Promise<NativeOutcome<null>> {
    const stream = this.#streams.get(id);
    if (stream === undefined) {
      if (this.#pending.has(id)) this.#cancelled.add(id);
      return ok(null);
    }
    this.#streams.delete(id);
    try { await stream.close(); return ok(null); } catch (error) { return failed(error); }
  }
}

"#;

const CUSTOM_CALLBACKS_PREFIX: &str = r#"class CustomCallbacks implements ForeignAdapterCallbacks {
  readonly #streams = new AdapterStreams();
  constructor(readonly adapter: Adapter) {}
  async dispatch(call: Wire.AdapterCallWire): Promise<NativeOutcome<Wire.AdapterReplyWire>> {
    try {
      switch (call.kind) {
"#;

const CUSTOM_CALLBACKS_SUFFIX: &str = r#"      }
    } catch (error) { return failed(error); }
  }
  streamNext(id: string) { return this.#streams.next(id); }
  streamClose(id: string) { return this.#streams.close(id); }
}

const nativeClients = new WeakMap<Adapter, NativeClientHandle>();
export function bindNativeClient(adapter: Adapter, native: NativeClientHandle): void {
  nativeClients.set(adapter, native);
}

async function* nativeItems<I, T>(handle: NativeStreamHandle<I>, decode: (value: I) => T): AsyncGenerator<T> {
  while (true) {
    const value = Codec.unwrapOutcome(await handle.next());
    if (value === null) return;
    yield decode(value);
  }
}

"#;

const BUILTIN_BASE_PREFIX: &str = r#"class NativeAdapter extends Adapter {
  readonly exchange: Model.Exchange;
  readonly features: ReadonlySet<Model.Feature>;
  protected constructor(native: NativeClientHandle) {
    super();
    const exchange = Model.Exchange.values.find((value) => value.id === native.exchange());
    if (exchange === undefined) throw new AdapterError(`unknown native exchange ${native.exchange()}`);
    this.exchange = exchange;
    this.features = new Set(Model.Feature.values.filter((feature) => native.supports(feature.id)));
    bindNativeClient(this, native);
  }
"#;

const BUILTIN_BASE_SUFFIX: &str = r#"}

export class BinanceListenKey {
  constructor(readonly id: string, readonly value: string) { Object.freeze(this); }
}

"#;
