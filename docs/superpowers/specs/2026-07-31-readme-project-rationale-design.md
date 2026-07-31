# README project rationale design

## Goal

Help Rust developers understand why `maxt` is convenient when one application
uses several exchanges. The section explains the user-facing benefit without
naming the internal product that motivated the library or comparing it with a
competitor.

## Placement

Add one short section after the opening paragraph and before the quick-start
section in both READMEs:

- `README.md`: `Why maxt`
- `README.ko.md`: `왜 maxt인가`

## Content

The section will state that:

- one application can use several exchanges through the same client methods;
- common ordering, time-range, numeric, missing-value, and error semantics keep
  application logic independent of provider response shapes;
- exchange-specific capabilities remain available through the typed adapter.

The copy will lead with convenience and keep implementation details in the
existing API reference.

## Exclusions

- Do not mention Youngcha or any other internal product.
- Do not mention CCXT or make comparative claims.
- Do not claim performance, latency, or private live-verification advantages.
- Do not repeat the supported-feature lists already covered elsewhere.

## Acceptance criteria

- English and Korean sections have the same structure and meaning.
- Every claim is supported by the current `Client`, common types, and adapter
  escape hatch.
- Existing README code examples and links remain unchanged.
- Markdown structure checks and documentation tests pass.
