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
      ]);
      async markets() { return []; }
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
    const depositAddress = await client.depositAddress(
      new maxt.DepositAddressRequest("BTC", maxt.Network.Bitcoin),
    );
    const quote = await client.prepareWithdrawal(withdrawalRequest);
    const withdrawal = await client.withdraw(withdrawalRequest);
    return {
      exchange: client.exchange.id,
      markets: (await client.markets(maxt.MarketKind.Spot)).length,
      supportsMarkets: client.supports(maxt.Feature.Markets),
      network: networks[0].network.id,
      depositAddress: depositAddress.address,
      withdrawalFee: quote.fee.toString(),
      withdrawal: withdrawal.id,
      deposits: (await client.deposits(historyRequest)).items.length,
      withdrawals: (await client.withdrawals(historyRequest)).items.length,
    };
  });

  expect(result).toEqual({
    exchange: "binance",
    markets: 0,
    supportsMarkets: true,
    network: "bitcoin",
    depositAddress: "bc1qdestination",
    withdrawalFee: "0.0001",
    withdrawal: "withdrawal-1",
    deposits: 0,
    withdrawals: 0,
  });
});
