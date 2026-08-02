from maxt._generated_contract import EXCHANGES, FEATURES
from maxt.models import Exchange, Feature


def test_generated_exchange_and_feature_inventories_match_public_models() -> None:
    assert EXCHANGES == tuple(value.value for value in Exchange)
    assert FEATURES == tuple(value.value for value in Feature)
