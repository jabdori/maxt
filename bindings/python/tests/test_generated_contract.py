from inspect import iscoroutinefunction

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


def test_generated_exchange_and_feature_inventories_match_public_models() -> None:
    assert EXCHANGES == tuple(value.value for value in Exchange)
    assert FEATURES == tuple(value.value for value in Feature)


def test_generated_api_inventories_match_public_classes() -> None:
    adapter_operations = tuple(
        name
        for name, value in Adapter.__dict__.items()
        if iscoroutinefunction(value)
    )
    assert ADAPTER_OPERATIONS == adapter_operations

    client_members = {
        name
        for name in Client.__dict__
        if not name.startswith("_") and name != "into_adapter"
    }
    assert set(CLIENT_MEMBERS) == client_members

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
