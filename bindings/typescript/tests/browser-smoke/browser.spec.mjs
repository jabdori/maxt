import { expect, test } from "@playwright/test";

test("browser export initializes WebAssembly and bridges a custom Adapter", async ({ page }) => {
  await page.goto("/");
  const result = await page.evaluate(async () => {
    const maxt = await import("/dist/browser.js");
    await maxt.initialize();

    class FixtureAdapter extends maxt.Adapter {
      exchange = maxt.Exchange.Binance;
      features = new Set([
        maxt.Feature.Markets,
        maxt.Feature.AssetNetworks,
        maxt.Feature.DepositAddresses,
        maxt.Feature.WithdrawalQuotes,
        maxt.Feature.Withdrawals,
        maxt.Feature.DepositHistory,
        maxt.Feature.WithdrawalHistory,
        maxt.Feature.OrderHistory,
        maxt.Feature.Trading,
      ]);
      async markets() { return []; }
      async orderRules(market) {
        return new maxt.OrderRules(
          market,
          "BTC/USDT",
          maxt.MarketStatus.Active,
          maxt.Decimal.parse("0.001"),
          maxt.Decimal.parse("0.001"),
          maxt.Decimal.parse("0.0005"),
          maxt.Decimal.parse("0.0005"),
          [maxt.Side.Buy, maxt.Side.Sell],
          [new maxt.OrderOption(
            "limit_ioc", maxt.OrderType.Limit, maxt.TimeInForce.ImmediateOrCancel,
          )],
          [new maxt.OrderOption("future_order", null, null)],
          maxt.Decimal.parse("0.1"),
          maxt.Decimal.parse("0.1"),
          maxt.Decimal.parse("10"),
          maxt.Decimal.parse("10"),
          maxt.Decimal.parse("1000000"),
          new maxt.OrderAccount(
            new maxt.Balance("USDT", maxt.Decimal.parse("100"), maxt.Decimal.zero),
            maxt.Decimal.zero, false, "USDT",
          ),
          new maxt.OrderAccount(
            new maxt.Balance("BTC", maxt.Decimal.one, maxt.Decimal.zero),
            maxt.Decimal.parse("50000"), false, "USDT",
          ),
        );
      }
      async assetNetworks(asset) {
        return [new maxt.AssetNetwork(
          this.exchange,
          asset,
          maxt.Network.Bitcoin,
          "BTC",
          true,
          true,
          maxt.WithdrawalFee.fixed(maxt.Decimal.parse("0.0001")),
          maxt.Decimal.parse("0.001"),
          null,
          false,
        )];
      }
      async depositAddress(request) {
        return new maxt.DepositAddress(
          this.exchange,
          request.asset,
          request.network,
          "bc1qdestination",
          null,
        );
      }
      async depositAddresses() {
        return [new maxt.DepositAddressEntry(
          this.exchange,
          "XRP",
          null,
          null,
          null,
          "tag-7",
        )];
      }
      async prepareWithdrawal() {
        return new maxt.WithdrawalQuote(
          maxt.Decimal.parse("0.0001"),
          maxt.Decimal.parse("0.9999"),
          null,
          null,
          true,
          maxt.TravelRuleRequirement.NotRequired,
          null,
        );
      }
      async withdraw(request) {
        return new maxt.Withdrawal(
          "withdrawal-1",
          request.asset,
          request.network,
          "BTC",
          request.amount,
          maxt.Decimal.parse("0.0001"),
          request.destination,
          maxt.WithdrawalStatus.Pending,
          "accepted",
          null,
          null,
        );
      }
      async deposits() { return new maxt.Page([], null); }
      async withdrawals() { return new maxt.Page([], null); }
      async order(market) {
        return new maxt.Order(
          "order-1", market, maxt.Side.Buy, maxt.OrderStatus.Filled,
          maxt.Decimal.one, maxt.Decimal.zero, maxt.Decimal.parse("100"), null,
        );
      }
      async orderByClientId(market) { return this.order(market); }
      async ordersByIds(request) { return [await this.order(request.market)]; }
      async orderHistory(request) {
        return new maxt.Page([await this.order(request.market)], null);
      }
      async cancelOrders(request) {
        return new maxt.CancelOrdersResult(
          [new maxt.CancelledOrder(
            request.ids[0], null, null, maxt.Timestamp.fromNanoseconds(125n),
          )],
          [new maxt.OrderCancelFailure(
            null, request.ids[1], null, "order_not_found", "not found",
          )],
        );
      }
    }

    const client = new maxt.Client(new FixtureAdapter());
    const destination = maxt.TransferDestination.chain(new maxt.ChainDestination(
      "BTC", maxt.Network.Bitcoin, "bc1qdestination",
    ));
    const withdrawalRequest = new maxt.WithdrawRequest(
      "BTC", maxt.Network.Bitcoin, maxt.Decimal.one, destination,
    );
    const historyRequest = new maxt.TransferHistoryRequest();
    const networks = await client.assetNetworks("BTC");
    const depositAddressEntry = (await client.depositAddresses())[0];
    const depositAddress = await client.depositAddress(
      new maxt.DepositAddressRequest("BTC", maxt.Network.Bitcoin),
    );
    const quote = await client.prepareWithdrawal(withdrawalRequest);
    const withdrawal = await client.withdraw(withdrawalRequest);
    const market = maxt.Market.spot(maxt.Exchange.Binance, "BTC", "USDT");
    const order = await client.order(market, "order-1");
    const rules = await client.orderRules(market);
    const orderByClientId = await client.orderByClientId(market, "client-1");
    const ordersByIds = await client.ordersByIds(
      new maxt.OrderLookupRequest(maxt.OrderIdKind.Exchange, ["order-1"], market),
    );
    const orderHistory = await client.orderHistory(
      new maxt.OrderHistoryRequest(market, [maxt.OrderStatus.Filled]),
    );
    const cancelResult = await client.cancelOrders(
      new maxt.CancelOrdersRequest(
        maxt.OrderIdKind.Client,
        ["client-1", "missing-1"],
      ),
    );
    const upbit = new maxt.UpbitAdapter();
    const upbitMarket = maxt.Market.spot(maxt.Exchange.Upbit, "BTC", "KRW");
    let aggregationError;
    try {
      await upbit.orderBooksAtLevel([upbitMarket], maxt.Decimal.parse("-1"));
    } catch (error) {
      aggregationError = {
        name: error.constructor.name,
        field: error.field,
      };
    }
    const bithumb = new maxt.BithumbAdapter();
    const noticeErrors = [];
    for (const count of [0, Number.NaN]) {
      try {
        await bithumb.notices(count);
      } catch (error) {
        noticeErrors.push({
          name: error.constructor.name,
          field: error.field,
        });
      }
    }
    let feeError;
    try {
      await bithumb.transferFees(" ");
    } catch (error) {
      feeError = {
        name: error.constructor.name,
        field: error.field,
      };
    }
    let apiKeysError;
    try {
      await bithumb.apiKeys();
    } catch (error) {
      apiKeysError = { name: error.constructor.name };
    }
    let pendingOrdersError;
    try {
      await bithumb.pendingOrders(new maxt.BithumbPendingOrdersRequest());
    } catch (error) {
      pendingOrdersError = { name: error.constructor.name };
    }
    return {
      exchange: client.exchange.id,
      markets: (await client.markets(maxt.MarketKind.Spot)).length,
      supportsMarkets: client.supports(maxt.Feature.Markets),
      network: networks[0].network.id,
      depositAddressEntry: {
        asset: depositAddressEntry.asset,
        network: depositAddressEntry.network,
        address: depositAddressEntry.address,
        memo: depositAddressEntry.memo,
      },
      depositAddress: depositAddress.address,
      withdrawalFee: quote.fee.toString(),
      withdrawal: withdrawal.id,
      deposits: (await client.deposits(historyRequest)).items.length,
      withdrawals: (await client.withdrawals(historyRequest)).items.length,
      order: order.id,
      orderRule: rules.buyOptions[0].timeInForce.id,
      orderByClientId: orderByClientId.id,
      ordersByIds: ordersByIds[0].id,
      orderHistory: orderHistory.items[0].id,
      cancelledAt: cancelResult.cancelled[0].cancelledAt.nanosecondsSinceEpoch.toString(),
      cancelFailure: cancelResult.failed[0].code,
      aggregationError,
      noticeErrors,
      feeError,
      apiKeysError,
      pendingOrdersError,
    };
  });

  expect(result).toEqual({
    exchange: "binance",
    markets: 0,
    supportsMarkets: true,
    network: "bitcoin",
    depositAddressEntry: {
      asset: "XRP",
      network: null,
      address: null,
      memo: "tag-7",
    },
    depositAddress: "bc1qdestination",
    withdrawalFee: "0.0001",
    withdrawal: "withdrawal-1",
    deposits: 0,
    withdrawals: 0,
    order: "order-1",
    orderRule: "immediate_or_cancel",
    orderByClientId: "order-1",
    ordersByIds: "order-1",
    orderHistory: "order-1",
    cancelledAt: "125",
    cancelFailure: "order_not_found",
    aggregationError: { name: "InvalidRequestError", field: "level" },
    noticeErrors: [
      { name: "InvalidRequestError", field: "count" },
      { name: "InvalidRequestError", field: "count" },
    ],
    feeError: { name: "InvalidRequestError", field: "currency" },
    apiKeysError: { name: "AuthError" },
    pendingOrdersError: { name: "AuthError" },
  });
});
