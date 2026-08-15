# Guide: browser WebAssembly and the relay boundary

[English](browser-relay.md) | [한국어](browser-relay.ko.md)

Use this guide when shipping the Dart Web or TypeScript browser binding.

## Public calls can be direct

Initialize the browser binding without a relay for a public market-data call.
The [Dart](../examples.md#browser-relay) and [TypeScript](../examples.md#browser-relay)
browser examples do this by default. Browser CORS and network policy still
apply.

## Signed calls need a relay and an explicit opt-in

When an adapter is configured with credentials or a private key in a browser,
initialize it with both a trusted `relayUrl` and the explicit insecure-browser
credential opt-in. The relay forwards signed HTTP requests and selected
WebSocket handshakes; it is not user authentication and it is not secret
storage.

The relay must sit behind a TLS ingress with application authentication and
rate limiting. Keep the upstream allowlists narrow. The exact deployment
variables, protocol, and security limits are in the [relay reference](../../relay/README.md).

## Keep the trust boundary visible

The SDK deliberately refuses browser credentials without this setup so a
constructor cannot silently turn a public web app into a key holder. If your
application does not need signed browser calls, do not deploy a relay.
