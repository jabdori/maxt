from dataclasses import fields, is_dataclass
from inspect import iscoroutinefunction
from typing import get_type_hints

from maxt import (
    Adapter,
    AdapterError,
    AuthError,
    BinanceAdapter,
    BithumbAdapter,
    Client,
    DecodeError,
    Exchange,
    ExchangeError,
    Feature,
    HyperliquidAdapter,
    InvalidRequestError,
    TransportError,
    UnsupportedError,
    UpbitAdapter,
)
from maxt._generated_contract import (
    ADAPTER_OPERATIONS,
    CLIENT_MEMBERS,
    ERROR_VARIANTS,
    EXCHANGES,
    FEATURES,
    PROVIDER_METHODS,
)
from maxt._generated_api import _GeneratedAdapterApi, _GeneratedClientApi
from maxt._generated_delegate import _GeneratedNativeClientDelegateApi
from maxt._generated_wire import ERROR_FIELDS, IDENTIFIER_VARIANTS, RECORD_FIELDS
from maxt import _generated_identifiers, models


def test_generated_exchange_and_feature_inventories_match_public_models() -> None:
    assert EXCHANGES == tuple(value.value for value in Exchange)
    assert FEATURES == tuple(value.value for value in Feature)

    ledger_kind = _generated_identifiers.HyperliquidLedgerKind
    assert ledger_kind.other("future_kind") is ledger_kind("future_kind")


def test_generated_api_inventories_match_public_classes() -> None:
    adapter_operations = tuple(
        name
        for name in ADAPTER_OPERATIONS
        if iscoroutinefunction(getattr(Adapter, name, None))
    )
    assert ADAPTER_OPERATIONS == adapter_operations
    assert set(ADAPTER_OPERATIONS) == {
        name
        for name, value in _GeneratedAdapterApi.__dict__.items()
        if iscoroutinefunction(value)
    }
    assert set(ADAPTER_OPERATIONS) == {
        name
        for name, value in _GeneratedNativeClientDelegateApi.__dict__.items()
        if iscoroutinefunction(value) and not name.startswith("_")
    }

    assert all(hasattr(Client, name) for name in CLIENT_MEMBERS)
    assert {
        name
        for name, value in _GeneratedClientApi.__dict__.items()
        if iscoroutinefunction(value)
    } == set(CLIENT_MEMBERS) - {"adapter", "exchange", "supports"}

    providers = {
        "upbit": (UpbitAdapter, set()),
        "bithumb": (BithumbAdapter, set()),
        "binance": (BinanceAdapter, {"spot", "usd_m_futures"}),
        "hyperliquid": (HyperliquidAdapter, {"testnet"}),
    }
    for exchange, (adapter, factories) in providers.items():
        members = {
            name
            for name in adapter.__dict__
            if not name.startswith("_") and name not in factories
        }
        assert set(PROVIDER_METHODS[exchange]) == members

    errors = (
        InvalidRequestError,
        UnsupportedError,
        AdapterError,
        AuthError,
        ExchangeError,
        TransportError,
        DecodeError,
    )
    assert ERROR_VARIANTS == tuple(error.__name__.removesuffix("Error") for error in errors)


def test_generated_runtime_annotations_resolve() -> None:
    for generated_type in (
        _GeneratedAdapterApi,
        _GeneratedClientApi,
        _GeneratedNativeClientDelegateApi,
    ):
        for value in generated_type.__dict__.values():
            if callable(value):
                get_type_hints(value)


def test_generated_wire_fields_match_public_models_and_errors() -> None:
    compared = 0
    for name, schema_fields in RECORD_FIELDS.items():
        model_type = getattr(models, name, None)
        if not isinstance(model_type, type) or not is_dataclass(model_type):
            continue
        actual = {
            item.metadata.get("wire_name", item.name) for item in fields(model_type)
        }
        assert actual == set(schema_fields), name
        compared += 1
    assert compared > 0

    errors = (
        InvalidRequestError,
        UnsupportedError,
        AdapterError,
        AuthError,
        ExchangeError,
        TransportError,
        DecodeError,
    )
    assert {error.kind for error in errors} == set(ERROR_FIELDS)

    for name, variants in IDENTIFIER_VARIANTS.items():
        identifier = getattr(_generated_identifiers, name)
        assert tuple(item.value for item in identifier) == variants
