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

  test('order history defaults to every final order', () {
    const request = OrderHistoryRequest();

    expect(request.market, isNull);
    expect(request.statuses, isEmpty);
  });

  test('Bithumb closed orders defaults to every final state', () {
    const request = BithumbClosedOrdersRequest();

    expect(request.market, isNull);
    expect(request.state, isNull);
    expect(request.states, isEmpty);
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
      'orderRules': adapter.orderRules,
      'assetNetworks': adapter.assetNetworks,
      'depositAddresses': adapter.depositAddresses,
      'depositAddress': adapter.depositAddress,
      'createDepositAddress': adapter.createDepositAddress,
      'prepareWithdrawal': adapter.prepareWithdrawal,
      'withdraw': adapter.withdraw,
      'deposit': adapter.deposit,
      'withdrawal': adapter.withdrawal,
      'cancelWithdrawal': adapter.cancelWithdrawal,
      'deposits': adapter.deposits,
      'withdrawals': adapter.withdrawals,
      'openOrders': adapter.openOrders,
      'order': adapter.order,
      'orderByClientId': adapter.orderByClientId,
      'ordersByIds': adapter.ordersByIds,
      'orderHistory': adapter.orderHistory,
      'subscribeAccount': adapter.subscribeAccount,
      'placeOrder': adapter.placeOrder,
      'cancelOrder': adapter.cancelOrder,
      'cancelOrderByClientId': adapter.cancelOrderByClientId,
      'cancelOrders': adapter.cancelOrders,
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
      'orderRules': (Client<Adapter> value) => value.orderRules,
      'assetNetworks': (Client<Adapter> value) => value.assetNetworks,
      'depositAddresses': (Client<Adapter> value) => value.depositAddresses,
      'depositAddress': (Client<Adapter> value) => value.depositAddress,
      'createDepositAddress': (Client<Adapter> value) =>
          value.createDepositAddress,
      'prepareWithdrawal': (Client<Adapter> value) => value.prepareWithdrawal,
      'withdraw': (Client<Adapter> value) => value.withdraw,
      'deposit': (Client<Adapter> value) => value.deposit,
      'withdrawal': (Client<Adapter> value) => value.withdrawal,
      'cancelWithdrawal': (Client<Adapter> value) => value.cancelWithdrawal,
      'deposits': (Client<Adapter> value) => value.deposits,
      'withdrawals': (Client<Adapter> value) => value.withdrawals,
      'prepareTransferTo': (Client<Adapter> value) => value.prepareTransferTo,
      'prepareTransferToChain': (Client<Adapter> value) =>
          value.prepareTransferToChain,
      'executeTransfer': (Client<Adapter> value) => value.executeTransfer,
      'openOrders': (Client<Adapter> value) => value.openOrders,
      'openOrdersOn': (Client<Adapter> value) => value.openOrdersOn,
      'order': (Client<Adapter> value) => value.order,
      'orderByClientId': (Client<Adapter> value) => value.orderByClientId,
      'ordersByIds': (Client<Adapter> value) => value.ordersByIds,
      'orderHistory': (Client<Adapter> value) => value.orderHistory,
      'subscribeAccount': (Client<Adapter> value) => value.subscribeAccount,
      'subscribeAccountWith': (Client<Adapter> value) =>
          value.subscribeAccountWith,
      'placeOrder': (Client<Adapter> value) => value.placeOrder,
      'cancelOrder': (Client<Adapter> value) => value.cancelOrder,
      'cancelOrderByClientId': (Client<Adapter> value) =>
          value.cancelOrderByClientId,
      'cancelOrders': (Client<Adapter> value) => value.cancelOrders,
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
        'orderBooksAtLevel': (UpbitAdapter value) => value.orderBooksAtLevel,
        'tickers': (UpbitAdapter value) => value.tickers,
        'tickersByQuote': (UpbitAdapter value) => value.tickersByQuote,
        'yearCandles': (UpbitAdapter value) => value.yearCandles,
        'orderbookInstruments': (UpbitAdapter value) =>
            value.orderbookInstruments,
        'marketEvents': (UpbitAdapter value) => value.marketEvents,
        'listSubscriptions': (UpbitAdapter value) => value.listSubscriptions,
        'testOrder': (UpbitAdapter value) => value.testOrder,
        'orderDetail': (UpbitAdapter value) => value.orderDetail,
        'closedOrders': (UpbitAdapter value) => value.closedOrders,
        'depositInfo': (UpbitAdapter value) => value.depositInfo,
        'withdrawalAddresses': (UpbitAdapter value) => value.withdrawalAddresses,
        'travelRuleVasps': (UpbitAdapter value) => value.travelRuleVasps,
        'verifyTravelRuleByUuid': (UpbitAdapter value) =>
            value.verifyTravelRuleByUuid,
        'verifyTravelRuleByTxid': (UpbitAdapter value) =>
            value.verifyTravelRuleByTxid,
        'batchCancelOpenOrders': (UpbitAdapter value) =>
            value.batchCancelOpenOrders,
        'cancelAndNewOrder': (UpbitAdapter value) => value.cancelAndNewOrder,
        'depositKrw': (UpbitAdapter value) => value.depositKrw,
        'withdrawKrw': (UpbitAdapter value) => value.withdrawKrw,
        'apiKeys': (UpbitAdapter value) => value.apiKeys,
        'listPockets': (UpbitAdapter value) => value.listPockets,
        'listPocketApiKeys': (UpbitAdapter value) => value.listPocketApiKeys,
        'subPocketBalances': (UpbitAdapter value) => value.subPocketBalances,
        'universalTransfer': (UpbitAdapter value) => value.universalTransfer,
        'universalTransfers': (UpbitAdapter value) => value.universalTransfers,
        'subPocketTransfer': (UpbitAdapter value) => value.subPocketTransfer,
        'subPocketTransfers': (UpbitAdapter value) => value.subPocketTransfers,
      },
      'bithumb': {
        'marketWarnings': (BithumbAdapter value) => value.marketWarnings,
        'marketAlerts': (BithumbAdapter value) => value.marketAlerts,
        'notices': (BithumbAdapter value) => value.notices,
        'transferFees': (BithumbAdapter value) => value.transferFees,
        'apiKeys': (BithumbAdapter value) => value.apiKeys,
        'krwWithdrawals': (BithumbAdapter value) => value.krwWithdrawals,
        'withdrawKrw': (BithumbAdapter value) => value.withdrawKrw,
        'krwDeposits': (BithumbAdapter value) => value.krwDeposits,
        'depositKrw': (BithumbAdapter value) => value.depositKrw,
        'pendingOrders': (BithumbAdapter value) => value.pendingOrders,
        'closedOrders': (BithumbAdapter value) => value.closedOrders,
        'batchOrders': (BithumbAdapter value) => value.batchOrders,
        'twapOrders': (BithumbAdapter value) => value.twapOrders,
        'createTwapOrder': (BithumbAdapter value) => value.createTwapOrder,
        'cancelTwapOrder': (BithumbAdapter value) => value.cancelTwapOrder,
        'withdrawalAddresses': (BithumbAdapter value) =>
            value.withdrawalAddresses,
        'orderDetail': (BithumbAdapter value) => value.orderDetail,
        'orderList': (BithumbAdapter value) => value.orderList,
      },
      'binance': {
        'venue': (BinanceAdapter value) => value.venue,
        'spotSymbolFilters': (BinanceAdapter value) => value.spotSymbolFilters,
        'spotOrder': (BinanceAdapter value) => value.spotOrder,
        'spotAveragePrice': (BinanceAdapter value) => value.spotAveragePrice,
        'spotAccountInformation': (BinanceAdapter value) =>
            value.spotAccountInformation,
        'spotCancelAllOpenOrders': (BinanceAdapter value) =>
            value.spotCancelAllOpenOrders,
        'spotExchangeInfo': (BinanceAdapter value) => value.spotExchangeInfo,
        'usdMAccountInformation': (BinanceAdapter value) =>
            value.usdMAccountInformation,
        'usdMExchangeInfo': (BinanceAdapter value) => value.usdMExchangeInfo,
        'usdMPositionInformation': (BinanceAdapter value) =>
            value.usdMPositionInformation,
        'allCoinsInformation': (BinanceAdapter value) => value.allCoinsInformation,
        'apiKeyPermissions': (BinanceAdapter value) => value.apiKeyPermissions,
        'depositHistory': (BinanceAdapter value) => value.depositHistory,
        'questionnaireRequirements': (BinanceAdapter value) =>
            value.questionnaireRequirements,
        'withdrawAddressList': (BinanceAdapter value) => value.withdrawAddressList,
        'withdrawHistory': (BinanceAdapter value) => value.withdrawHistory,
        'markPrice': (BinanceAdapter value) => value.markPrice,
        'markPrices': (BinanceAdapter value) => value.markPrices,
        'openInterest': (BinanceAdapter value) => value.openInterest,
        'aggregateTrades': (BinanceAdapter value) => value.aggregateTrades,
        'accountTrades': (BinanceAdapter value) => value.accountTrades,
        'c2cTradeHistory': (BinanceAdapter value) => value.c2cTradeHistory,
        'testOrder': (BinanceAdapter value) => value.testOrder,
        'cancelAllOpenOrders': (BinanceAdapter value) =>
            value.cancelAllOpenOrders,
        'usdMCreateListenKey': (BinanceAdapter value) =>
            value.usdMCreateListenKey,
        'usdMKeepaliveListenKey': (BinanceAdapter value) =>
            value.usdMKeepaliveListenKey,
        'usdMCloseListenKey': (BinanceAdapter value) =>
            value.usdMCloseListenKey,
      },
      'hyperliquid': {
        'isTestnet': (HyperliquidAdapter value) => value.isTestnet,
        'allMids': (HyperliquidAdapter value) => value.allMids,
        'userFills': (HyperliquidAdapter value) => value.userFills,
        'userFillsByTime': (HyperliquidAdapter value) => value.userFillsByTime,
        'basicOpenOrders': (HyperliquidAdapter value) => value.basicOpenOrders,
        'orderStatus': (HyperliquidAdapter value) => value.orderStatus,
        'historicalOrders': (HyperliquidAdapter value) => value.historicalOrders,
        'nonFundingLedger': (HyperliquidAdapter value) =>
            value.nonFundingLedger,
        'assetContext': (HyperliquidAdapter value) => value.assetContext,
        'candleSnapshot': (HyperliquidAdapter value) => value.candleSnapshot,
        'l2Book': (HyperliquidAdapter value) => value.l2Book,
        'recentTrades': (HyperliquidAdapter value) => value.recentTrades,
        'fundingHistory': (HyperliquidAdapter value) => value.fundingHistory,
        'userFunding': (HyperliquidAdapter value) => value.userFunding,
        'spotClearinghouseState': (HyperliquidAdapter value) =>
            value.spotClearinghouseState,
        'spotMeta': (HyperliquidAdapter value) => value.spotMeta,
        'spotMetaAndAssetContexts': (HyperliquidAdapter value) =>
            value.spotMetaAndAssetContexts,
        'userRateLimit': (HyperliquidAdapter value) => value.userRateLimit,
        'userRole': (HyperliquidAdapter value) => value.userRole,
        'referral': (HyperliquidAdapter value) => value.referral,
        'userFees': (HyperliquidAdapter value) => value.userFees,
        'portfolio': (HyperliquidAdapter value) => value.portfolio,
        'subAccounts': (HyperliquidAdapter value) => value.subAccounts,
        'userVaultEquities': (HyperliquidAdapter value) =>
            value.userVaultEquities,
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
