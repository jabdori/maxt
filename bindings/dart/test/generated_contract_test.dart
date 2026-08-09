import 'package:test/test.dart';
import 'package:maxt/maxt.dart';

import '../lib/src/generated_contract.dart';

final class InventoryAdapter extends AdapterBase {
  @override
  Exchange get exchange => Exchange.upbit;

  @override
  Set<Feature> get features => const {};
}

void main() {
  test('generated exchange and feature inventories match public models', () {
    expect(exchanges, Exchange.values.map((value) => value.id));
    expect(features, Feature.values.map((value) => value.wireName));
  });

  test('generated public API inventories stay explicit', () {
    final adapter = InventoryAdapter();
    final actualAdapter = <String, Object>{
      'markets': adapter.markets,
      'trades': adapter.trades,
      'orderBook': adapter.orderBook,
      'ticker': adapter.ticker,
      'candles': adapter.candles,
      'subscribe': adapter.subscribe,
      'balances': adapter.balances,
      'assetNetworks': adapter.assetNetworks,
      'depositAddress': adapter.depositAddress,
      'prepareWithdrawal': adapter.prepareWithdrawal,
      'withdraw': adapter.withdraw,
      'deposits': adapter.deposits,
      'withdrawals': adapter.withdrawals,
      'openOrders': adapter.openOrders,
      'subscribeAccount': adapter.subscribeAccount,
      'placeOrder': adapter.placeOrder,
      'cancelOrder': adapter.cancelOrder,
      'positions': adapter.positions,
      'marginSummary': adapter.marginSummary,
      'fundingRates': adapter.fundingRates,
      'fundingPayments': adapter.fundingPayments,
      'setMargin': adapter.setMargin,
    };
    expect(actualAdapter.keys, adapterOperations);

    final actualClient = <String, Object>{
      'exchange': (Client<Adapter> value) => value.exchange,
      'supports': (Client<Adapter> value) => value.supports,
      'adapter': (Client<Adapter> value) => value.adapter,
      'markets': (Client<Adapter> value) => value.markets,
      'trades': (Client<Adapter> value) => value.trades,
      'orderBook': (Client<Adapter> value) => value.orderBook,
      'ticker': (Client<Adapter> value) => value.ticker,
      'candles': (Client<Adapter> value) => value.candles,
      'subscribe': (Client<Adapter> value) => value.subscribe,
      'subscribeWith': (Client<Adapter> value) => value.subscribeWith,
      'balances': (Client<Adapter> value) => value.balances,
      'assetNetworks': (Client<Adapter> value) => value.assetNetworks,
      'depositAddress': (Client<Adapter> value) => value.depositAddress,
      'prepareWithdrawal': (Client<Adapter> value) => value.prepareWithdrawal,
      'withdraw': (Client<Adapter> value) => value.withdraw,
      'deposits': (Client<Adapter> value) => value.deposits,
      'withdrawals': (Client<Adapter> value) => value.withdrawals,
      'prepareTransferTo': (Client<Adapter> value) => value.prepareTransferTo,
      'prepareTransferToChain': (Client<Adapter> value) =>
          value.prepareTransferToChain,
      'executeTransfer': (Client<Adapter> value) => value.executeTransfer,
      'openOrders': (Client<Adapter> value) => value.openOrders,
      'openOrdersOn': (Client<Adapter> value) => value.openOrdersOn,
      'subscribeAccount': (Client<Adapter> value) => value.subscribeAccount,
      'subscribeAccountWith': (Client<Adapter> value) =>
          value.subscribeAccountWith,
      'placeOrder': (Client<Adapter> value) => value.placeOrder,
      'cancelOrder': (Client<Adapter> value) => value.cancelOrder,
      'positions': (Client<Adapter> value) => value.positions,
      'positionsOn': (Client<Adapter> value) => value.positionsOn,
      'marginSummary': (Client<Adapter> value) => value.marginSummary,
      'fundingRates': (Client<Adapter> value) => value.fundingRates,
      'fundingPayments': (Client<Adapter> value) => value.fundingPayments,
      'setMargin': (Client<Adapter> value) => value.setMargin,
    };
    expect(actualClient.keys, clientMembers);

    final actualProviders = <String, Map<String, Object>>{
      'upbit': {
        'region': (UpbitAdapter value) => value.region,
        'orderBooks': (UpbitAdapter value) => value.orderBooks,
        'tickers': (UpbitAdapter value) => value.tickers,
        'marketEvents': (UpbitAdapter value) => value.marketEvents,
      },
      'bithumb': {
        'marketWarnings': (BithumbAdapter value) => value.marketWarnings,
        'marketAlerts': (BithumbAdapter value) => value.marketAlerts,
      },
      'binance': {
        'venue': (BinanceAdapter value) => value.venue,
        'spotSymbolFilters': (BinanceAdapter value) => value.spotSymbolFilters,
        'spotOrder': (BinanceAdapter value) => value.spotOrder,
        'usdMCreateListenKey': (BinanceAdapter value) =>
            value.usdMCreateListenKey,
        'usdMKeepaliveListenKey': (BinanceAdapter value) =>
            value.usdMKeepaliveListenKey,
        'usdMCloseListenKey': (BinanceAdapter value) =>
            value.usdMCloseListenKey,
      },
      'hyperliquid': {
        'isTestnet': (HyperliquidAdapter value) => value.isTestnet,
        'nonFundingLedger': (HyperliquidAdapter value) =>
            value.nonFundingLedger,
        'assetContext': (HyperliquidAdapter value) => value.assetContext,
      },
    };
    expect(
      actualProviders.map((key, value) => MapEntry(key, value.keys.toList())),
      providerMethods,
    );

    final actualErrors = <String, Object>{
      'InvalidRequest': InvalidRequestError.new,
      'Transfer': TransferError.new,
      'Unsupported': UnsupportedError.new,
      'Adapter': AdapterError.new,
      'Auth': AuthenticationError.new,
      'Exchange': ExchangeError.new,
      'Transport': TransportError.new,
      'Decode': DecodeError.new,
    };
    expect(actualErrors.keys, errorVariants);
  });
}
