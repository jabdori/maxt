[English](CONTRIBUTING.md) | [한국어](CONTRIBUTING.ko.md)

# Contributing to maxt

`maxt` puts one Rust API in front of four exchanges. Two questions come up
constantly: where a feature belongs, and what adding an exchange requires.

## Setup

Rust 1.85 or newer, edition 2024. Both are pinned in `Cargo.toml`.

```sh
git clone https://github.com/jabdori/maxt
cd maxt
cargo test
```

No exchange account and no connection to an exchange are needed. Every check
in the list below runs offline, and an adapter that had to open a connection to
answer something it already knows would be a bug. One check is deliberately
outside that list and does open connections: see
[The live conformance check](#the-live-conformance-check).

## The checks

`.github/workflows/ci.yml` runs these on every push and pull request; run them
before opening one. CI exports `RUSTFLAGS: -D warnings` for the whole job, so
export it locally too, or a warning that fails CI passes here unnoticed.

```sh
export RUSTFLAGS="-D warnings"

cargo fmt --all --check                    # formatting
cargo clippy --all-targets -- -D warnings  # lints, including tests and examples
cargo test --all-targets                   # unit and integration tests
cargo test --doc                           # doc tests, which --all-targets skips
cargo build --examples                     # the runnable programs still compile
cargo doc --no-deps                        # docs build, and intra-doc links resolve
```

One more, enforcing a house rule clippy's defaults do not. `--lib` builds
without `cfg(test)`, so only shipping code is checked. It must exit clean.

```sh
cargo clippy --lib -- -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic
```

One check is deliberately absent from that list. `tests/live_conformance.rs`
opens a socket to every exchange, so it is `#[ignore]`d and no unattended run
reaches the network. See
[The live conformance check](#the-live-conformance-check).

A second job, `scope`, greps the tree, Markdown and Rust and TOML, for two
things that must never land and fails on either. Both are impossible to undo
once pushed to a public remote.

| Rejected | Instead |
| --- | --- |
| Any claim that `maxt` is published to a package registry: an install command that would fetch it from one, a registry link for it, a hosted API-documentation link for it. | Depend on the repository, the way the README and [Getting started](docs/getting-started.md) do. |
| An absolute path out of a contributor's home directory. | Paths in prose, comments, and configuration are relative to the repository root. |

## Where a feature goes

Every feature lands in exactly one of three places. Something belongs on the
common API when every exchange can carry it without
changing what it means. It belongs on the adapter when the common API can only
carry it by stating something untrue or by throwing away the thing that made it
worth calling. It belongs nowhere when the exchange does not offer it.

Verify that last one against the exchange's own endpoint list before concluding
it. This repository has shipped a false absence more than once, and a reader
restructures their program around one.

| Destination | What it is |
| --- | --- |
| The common API | [`Adapter`](src/adapter.rs) declares the method, [`Client`](src/client.rs) forwards it, [`Feature`](src/feature.rs) names it. |
| An inherent method on the adapter | A `pub` method on `UpbitAdapter`, `BinanceAdapter`, and so on, reached through [`Client::adapter`](src/client.rs): `client.adapter().order_books(&markets, Some(5))`. |
| Nowhere, the trait default | Do not implement the method at all. `Adapter` defaults every method to `Error::Unsupported`, so an absent feature is reported at the call, by name, without the adapter writing a line. |

Every exchange-specific method in the codebase already carries a doc comment
naming the meaning that would have been lost. Read them before you add another.

| Method | Why it could not be common |
| --- | --- |
| `UpbitAdapter::order_books`, `UpbitAdapter::tickers` | Upbit answers for many markets in one request. Routed through `Client::order_book`, thirty markets become thirty calls, which against Upbit's per-second quota is the entire reason to call it. Neither method caps the list length and Upbit publishes no cap, so a long enough list is refused upstream as `Error::Exchange`. |
| `BithumbAdapter::market_warnings` | Bithumb designates a market for investment warning, 유의 종목, while leaving it tradable. `MarketStatus` has no value meaning "trading, but flagged", so `Client::markets` reports `MarketStatus::Unknown` and the label stays here verbatim. |
| `BithumbAdapter::market_alerts` | Bithumb's other designation, 주의 종목, is on a separate endpoint and carries a criterion, a severity step and an expiry. `MarketStatus` holds none of the three, and the designation never moves a market off `Active`. |
| `BinanceAdapter::spot_symbol_filters` | Tick size, lot step, and minimum notional decide whether an order is accepted at all, and no two exchanges express them alike. The type stays Binance-shaped. |
| `BinanceAdapter::spot_order` | Answers for filled and cancelled orders, which `Client::open_orders` by definition does not. |
| `BinanceAdapter::usd_m_create_listen_key` and its keepalive/close pair | `Client::subscribe_account` already runs this lifecycle. These exist for driving the socket yourself: sharing a key across consumers, or holding one across a restart. |
| `HyperliquidAdapter::non_funding_ledger` | Deposits, withdrawals, transfers, and liquidations belong to no market. Reporting one as a `FundingPayment` would have to name a market it never touched. |
| `HyperliquidAdapter::asset_context` | `FundingRate` records what funding *was* charged. What the next charge is running at is a different question, and open interest and oracle price have no common counterpart at all. |

Adding one means writing that sentence for it. If the honest answer is "it would
fit fine, I just wrote it here first", it belongs on the common API instead, on
every adapter.

### `supports()` has to be true

`Adapter::supports` is what feature checks, routing logic, and the provider
documentation all read. A `true` that then fails is worse than a `false`,
because callers branch on it: one who reads `false` picks another exchange, one
who reads `true` writes code that dies at runtime.

[`tests/unsupported_is_honest.rs`](tests/unsupported_is_honest.rs) is the
invariant, and it runs over the whole `Feature` by adapter-configuration cross
product in both directions:

- `supports(f) == false` means the call refuses as `Error::Unsupported` naming
  that same `f`: not a transport error, not a success, and not a different
  feature. A missing credential is the one other honest refusal, and it is
  `Error::Auth`.
- `supports(f) == true` means the call never answers `Unsupported`. This
  direction is harder offline, because a feature an adapter really has is
  usually answered by the exchange, so `offline_probe` uses whatever resolves
  before the wire: a market belonging to another exchange, or a malformed wallet
  address. Where nothing resolves it returns `None` and says why, and a floor on
  the probe count keeps that from quietly emptying the test.

Nothing there touches the network, so an adapter that reached for the wire to
report something it already knows would hang the suite.

What it does not cover is an argument to a call rather than the call itself.
`Feature::Candles` is `true` on every adapter and a candle interval outside that
exchange's set is still `Unsupported`. Keep the intervals honest with the
separate `every_baseline_interval_is_mapped_on_the_exchanges_that_can_be_asked_offline`,
whose `BASELINE_INTERVALS` is read off the four exchanges' own documentation and
not off the adapters. A baseline copied from what the adapters happen to
implement would assert the code against itself.

The name is that long on purpose. The probe is a market from another exchange,
and only an adapter that rejects one before opening a connection can answer
offline; Hyperliquid builds its symbol table first, so it is skipped there and
its interval map is asserted in its own unit tests. A name claiming all four
exchanges would be the overstatement the file exists to catch. Keep it that way
when you rename it.

Write `supports()` off the two helpers on `Feature`, not by hand, and comment
every exception. Note the credential gate: `supports()` answers for the adapter
as configured, so one built without credentials reports `false` for everything
that needs them.

```rust
use maxt::Feature;

struct ExampleAdapter {
    credentials: Option<(String, String)>,
}

impl ExampleAdapter {
    fn supports(&self, feature: Feature) -> bool {
        if feature.is_derivatives_only() {
            return false;
        }
        // Bithumb's public WebSocket carries trades, order books, and tickers,
        // but no candles.
        if matches!(feature, Feature::CandleStream) {
            return false;
        }
        if feature.needs_credentials() {
            return self.credentials.is_some();
        }
        true
    }
}

fn check() {
    let public = ExampleAdapter { credentials: None };
    let keyed = ExampleAdapter {
        credentials: Some(("access".to_string(), "secret".to_string())),
    };

    // Public market data is open to both, and the one commented exception
    // stays shut for both.
    assert!(public.supports(Feature::Ticker));
    assert!(!public.supports(Feature::CandleStream));
    assert!(!keyed.supports(Feature::CandleStream));

    // The credential gate, and the derivatives gate that a key cannot open.
    assert!(!public.supports(Feature::Balances));
    assert!(keyed.supports(Feature::Balances));
    assert!(!keyed.supports(Feature::Positions));
}
```

One thing that is not an unsupported feature: a spot exchange asked to list
perpetuals returns an empty list, not an error.

## Adding an exchange

`src/adapters/bithumb/` is the smallest of the four and has the same shape as
the rest. Read it end to end before starting.

1. **Add the `Exchange` variant** in `src/types/market.rs`. `Exchange::id` and
   `Exchange::display_name` are exhaustive matches, so the compiler will name
   every place that has to change.

2. **Create `src/adapters/<name>/`, one file per concern.** All four adapters
   follow the same split. Split further only when there is a reason: Hyperliquid
   signs with a wallet key rather than a header, so its signing lives in
   `sign.rs`, and its two exchange-shaped public types live in `native.rs`.

   | File | Holds |
   | --- | --- |
   | `mod.rs` | the adapter type, its constructors and credentials, and `impl Adapter` |
   | `rest.rs` | public REST: request builders returning `HttpRequest`, and the calls that send them |
   | `private.rs` | signed REST calls and the signing they need |
   | `stream.rs` | WebSocket subscribe frames and frame decoding |
   | `parse.rs` | the exchange's payload types and the conversion into `maxt` types |

3. **Register it in `src/adapters/mod.rs`**: a private `mod <name>;` and a
   `pub use` for the adapter plus any exchange-shaped public types it returns.

4. **Implement `Adapter`.** `exchange()` and `supports()` are required.
   Implement only the methods the exchange actually has and leave the rest at
   the default. Constructors are infallible, because callers expect them to be:
   building the HTTP transport can fail, but only if the TLS backend refuses to
   initialize, so keep that failure inside the type and report it at the first
   call that needs the network, the way the existing adapters do.

   ```rust
   use maxt::{Adapter, BoxFuture, Exchange, Feature, MarketInfo, MarketKind, Result};

   struct ExampleAdapter;

   impl Adapter for ExampleAdapter {
       fn exchange(&self) -> Exchange {
           Exchange::Upbit
       }

       fn supports(&self, feature: Feature) -> bool {
           matches!(feature, Feature::Markets)
       }

       fn markets(&self, kind: MarketKind) -> BoxFuture<'_, Result<Vec<MarketInfo>>> {
           let _ = kind;
           Box::pin(async move { Ok(Vec::new()) })
       }
   }

   fn check() {
       assert_eq!(ExampleAdapter.exchange(), Exchange::Upbit);
       assert!(ExampleAdapter.supports(Feature::Markets));
       // Everything left at the trait default reports itself absent, by name,
       // without this adapter writing a line about it.
       assert!(!ExampleAdapter.supports(Feature::Ticker));
   }
   ```

5. **Go through `src/transport/`**, not through an HTTP or WebSocket client:
   `HttpTransport` and `HttpRequest` for REST, `ws::connect` for sockets.
   Reconnect, heartbeat, and the caller's overflow policy live in
   `src/transport/ws.rs` so that four adapters do not each reimplement them, and
   reaching for `reqwest` or `tokio_tungstenite` directly gives up all of it.
   Build requests as plain functions returning `HttpRequest`. That is what lets
   every path, query, and rejection be tested without a network.

6. **Write the tests**, in `#[cfg(test)] mod tests` beside the code:

   | Test | What it has to do |
   | --- | --- |
   | Parse | Real payloads. Inline the exchange's own documented example as a `const` string with the documentation URL in a comment directly above it. `src/adapters/upbit/parse.rs` has twelve, eleven of them carrying a URL; the twelfth is an error body Upbit no longer publishes a reference page for, and the comment in place of the URL says so. Never paste a response from your own account. |
   | Request building | Path, query, and that an out-of-range `limit` is rejected before the request is built. |
   | Signing vector | Prefer the exchange's own published worked example: `binance/private.rs` checks against the key, query, and signature Binance documents, and `hyperliquid/sign.rs` checks that the documented key derives the documented address. Where the exchange publishes none, verify the way the exchange would, as `upbit/private.rs` does by decoding its own JWT back and checking that it verifies under the signing secret and fails under a different one. |
   | `supports()` | One per group: derivatives declined by a spot venue, the private half opening only with credentials, public market data open without them. |
   | Private calls fail before the network | Every account call on an adapter with no credentials returns `Error::Auth`. |
   | `tests/unsupported_is_honest.rs` | Not written here. It is a repository-level test with its own registration point; step 7 covers it. |

7. **Register it in [`tests/unsupported_is_honest.rs`](tests/unsupported_is_honest.rs).**
   Almost every test in that file iterates one list, so there are two places to
   touch and no obligatory per-adapter test to write.

   Add a constructor beside `upbit()`, `bithumb()`, `binance()` and
   `hyperliquid()`, returning a `Case`, then push each configuration of it onto
   `every_configuration()`. Anonymous and credentialed are separate cases, and
   both belong there; `binance()` shows a venue split as well.

   | `Case` field | What to put in it |
   | --- | --- |
   | `name` | what a failure names, one per configuration: `"upbit"` and `"upbit+keys"` |
   | `client` | the adapter through `boxed(..)`, so every case has one type |
   | `market` | a market this exchange actually lists |
   | `elsewhere` | a market it does not, for probes that must stop before the wire |
   | `checks_markets_offline` | whether it rejects an unlisted market without opening a connection. `false` for Hyperliquid, which builds its symbol table first |
   | `checks_credentials_offline` | whether a malformed credential is rejected before a connection. `false` unless the credential has a shape to be wrong about, as Hyperliquid's wallet address does |
   | `credentialed` | whether this configuration was given credentials at all |

   Then add the adapter to the four-element array in
   `missing_credentials_read_the_same_way_on_every_exchange`, which is the one
   list in the file that is not `every_configuration()`. Its length is in the
   type, so the compiler will point at it.

   Nothing else needs a new test. There is no per-adapter derivatives test:
   `a_feature_an_adapter_declines_is_declined_by_the_call_behind_it` already
   covers every derivatives feature on every spot-only configuration, and
   `every_private_feature_is_closed_until_credentials_are_supplied` walks the
   same list rather than an adapter list of its own. Write one of your own only
   where one exchange answers a shared call differently enough to be worth
   stating in a name, the way
   `hyperliquid_serves_recent_trades_over_rest_as_well_as_live` does.

   Name what you checked against the exchange, never an absence inferred from its
   documentation. That test asserted the opposite for a while, under a name that
   read as a certified invariant, and the endpoint had been live the whole time:
   it is missing from Hyperliquid's info reference and named on its rate-limit
   page. A missing feature is worth a test only once a request to the exchange has
   come back saying so.

   Adding a `Feature` variant is the other change that file needs, and it is
   unrelated to adding an exchange: `ALL_FEATURES` is a fixed-length array and
   `call` has a match arm per feature, so both fail to compile until the new
   variant is wired up.

8. **Add a provider page in `docs/providers/`, English and Korean.** Same claims
   in both, and every claim about what the exchange supports has to match
   `supports()`.

9. **Check the examples still build** with `cargo build --examples`. Add the
   exchange to one of them where it fits. Do not add an example per exchange.

## House rules

Enforced, not requested.

| Rule | Enforced by |
| --- | --- |
| No `unwrap`, `expect`, or `panic!` outside test modules. Return an `Error`. | the `cargo clippy --lib` command above |
| `unsafe` is forbidden. | `[lints.rust] unsafe_code = "forbid"` in `Cargo.toml`: the compiler rejects it, and `allow` cannot override `forbid`. |
| Every public item is documented. | `missing_docs = "warn"` in `Cargo.toml` |
| Public enums carry `#[non_exhaustive]`, so a variant can be added later without breaking callers. | review |
| Money is `rust_decimal::Decimal` and never `f64`. | review |
| Comments explain why, not what. | review |

**Decimal.** Prices, quantities, and amounts lose digits through a float.
`1386929.37231066771348207123` and `30854658886.18521` are both in the test
suite precisely because they do not survive an `f64`. `serde_json` is configured
with `arbitrary_precision`, so a `serde_json::Number` still holds the digits the
exchange sent; build the `Decimal` from that text. `grep -rn f64 src/` turns up
comments explaining this and nothing else. No `f64` sits on the path an
exchange payload takes, and none should.

**Comments.** The code already says what it does. Comments carry what it cannot:
why a request is shaped that way, what the exchange does that forced it, what
breaks if it changes. `src/adapter.rs` explains why `unsupported` is a free
function and not a trait method; `binance/mod.rs` explains why `BTCUSDT` is
split against a table of quote assets and not at a fixed offset.

## Testing against a live exchange

Everything public works without credentials, on all four exchanges. Reach for an
account only when you are changing a signed path. None of it belongs in the test
suite: the repository's tests run offline, and that is a property worth keeping.

| Exchange | Testnet |
| --- | --- |
| Hyperliquid | Yes, and `maxt` supports it. A separate host and a separate signing domain. |
| Binance | Binance publishes one, `maxt` does not wire it up. `BinanceMarket::rest_base_url` returns the production hosts and there is no override, so a Binance credential in `maxt` today acts on a real account. Testnet constructors would be a welcome change. |
| Upbit, Bithumb | None published at all. Their private paths run against a real account with real money. |

```rust
use maxt::Client;
use maxt::adapters::HyperliquidAdapter;

let client = Client::new(HyperliquidAdapter::testnet().with_wallet(
    "0x0000000000000000000000000000000000000000",
    "0x0123456789012345678901234567890123456789012345678901234567890123",
));
assert!(client.adapter().is_testnet());
```

A testnet signature does not recover on a mainnet digest, which
`hyperliquid/sign.rs::mainnet_and_testnet_signatures_are_not_interchangeable`
asserts, so you cannot accidentally aim one at the other. Prefer an approved API
wallet key over the account's own key: it can trade but cannot withdraw. On
Upbit and Bithumb, use a key restricted to the narrowest permissions that let
you reproduce what you are working on.

### The live conformance check

One command, and the only thing in the repository that opens a connection:

```sh
cargo test --test live_conformance -- --ignored --nocapture
```

`tests/live_conformance.rs` carries `#[ignore]`, so `cargo test`, `cargo test
--all-targets`, and CI all compile it and skip it. Nothing reaches an exchange
until that command is typed.

It works out its own subject list. Every exchange configuration is asked
`Client::supports` about the four streaming features, and each `true` becomes
one subscription; the depth it holds a book stream to is read out of the
`Feed::OrderBook` row of that exchange's provider page rather than restated in
the test. A feed an adapter starts claiming is therefore checked by the commit
that claims it, and a page that changes its promised depth changes what is
asserted.

| Claim | Checked as |
| --- | --- |
| The feed is carried | a nonzero count of that feed's own event type. A successful subscribe says only that a socket opened |
| The feed decodes | zero `Err` items. A feed that yields nothing but errors reads as supported everywhere else |
| A candle stream settles | at least one event with `Candle::closed`, over two `Min1` window boundaries |
| A book stream is as deep as its page says | every event carrying the levels a side the provider page states |
| A clock is a clock | every timestamp between five minutes behind the reading machine and thirty seconds ahead, on the streams and on the three public REST reads that carry one |

The wall-clock window is what catches a field read at the wrong scale, a field
carrying a local wall clock in a UTC slot, and a machine whose own clock is
wrong. All three look correct to a unit test holding a hand-written fixture.

| Subject | Figure |
| --- | --- |
| Runtime | about three minutes: 150 seconds with every subscription open at once, plus connect time and the REST reads |
| Why 150 seconds | a `Min1` window has to open and close while the check is watching, and 150 seconds crosses two boundaries wherever in a minute it starts |
| Connections | one per `Feed` and exchange pair, so that an error belongs to a feed rather than to a socket carrying several |
| Markets | the busiest market on each venue, so that a count of zero means a dead feed and not a quiet hour |
| Credentials | none are read, and none would be used |

A clean run ends `24 of 24 checks passed`. **Every pair is expected to pass, and
no row is allowed to stand red.** A red row is a regression until you have shown
otherwise, which is the only thing that makes the check worth running: a row
known to fail is a row nobody reads.

Each pair prints a line, and a failure names the pair and the numbers behind it.
These two are the defects that got through a full green unit suite:

```text
bithumb OrderBook          FAIL  0 events, 1087 errors, levels none; no event of this feed's own kind arrived; first error: could not read exchange response: `timestamp` is not a millisecond timestamp: 1785401551669967
hyperliquid Candles(Min1)  FAIL  125 events, 0 errors, 0 settled, clock off by 61423ms at worst; no Min1 candle closed across 150s and at least two window boundaries
```

On the second line the defect is `0 settled`. Read the clause, not the largest
number: `clock off by 61423ms` is inside the window this check allows, and the
same feed prints figures like it on rows marked `ok`.

| A failure saying | Read it as |
| --- | --- |
| `N errors, first error: ...` | the frames arrive and `maxt` cannot read them. A parse bug, or a payload the exchange changed |
| `0 events, 0 errors` | the subscribe was accepted and nothing came. Suspect the endpoint before the network: Binance USD-M acknowledges a `SUBSCRIBE` for a stream the entry point it is on does not carry, and then sends nothing for it forever, with no error and no close. That is what made `Ticker` and `Candles(Min1)` read as dead on USD-M until the adapter routed them to [the entry point that carries them](docs/providers/binance.md#the-two-usd-m-entry-points). Ask the same subject over REST and count frames on a raw socket before calling it a `maxt` defect |
| `0 settled` | the frames arrive but no window is ever announced finished. `Candle::closed` is a promise nothing else checks |
| `clock off by ...` | a timestamp is far from the wall clock. Check the machine's own clock first, then the field's documented scale |

### What it does not cover

| Not covered | Why |
| --- | --- |
| The private half: balances, open orders, `place_order`, `cancel_order`, `subscribe_account` | public read-only endpoints only. Signing needs a real key, and Upbit and Bithumb publish no testnet, so an order check would spend real money to answer a question |
| Hyperliquid's testnet | the same adapter aimed at another host. Its books and candles are thin enough that a count of zero would say nothing |
| Every market but one per venue, and every interval but `Min1` | one liquid market and the shortest common interval are what make a zero count and a window boundary meaningful inside three minutes |
| REST beyond `ticker`, `order_book`, and `trades` | those three carry the clocks. Paging, market lists, and funding history are checked offline |
| Whether a price or a size is right | it counts events and reads clocks. It does not know what BTC costs |
| That a feed stays alive | it reports the three minutes it ran for, and nothing about the next hour |

## What never goes in a commit

- API keys, secret keys, wallet private keys, JWTs, listen keys.
- A signed request captured off the wire. The signature is derived from the
  secret and the request together.
- Any response from a real account: balances, order histories, ledger entries,
  addresses. Test payloads start from the exchange's own documented example, and
  each one carries that page's URL beside it. Editing a documented payload is
  fine where a test needs two fields to differ that the example happens to make
  equal; say so in the comment, as `binance/stream.rs` does.
- `.env` files. `.gitignore` already excludes `/.env`, `/.env.*`, `*.pem`, and
  `*.key`; do not defeat it.

If a credential has already been pushed, rotate it at the exchange first. Git
history is not a place secrets can be removed from.
