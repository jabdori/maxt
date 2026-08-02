import { expect, test } from "@playwright/test";

test("browser export initializes WebAssembly and bridges a custom Adapter", async ({ page }) => {
  await page.goto("/");
  const result = await page.evaluate(async () => {
    const maxt = await import("/dist/browser.js");
    await maxt.initialize();

    class EmptyAdapter extends maxt.Adapter {
      exchange = maxt.Exchange.Binance;
      features = new Set([maxt.Feature.Markets]);
      async markets() { return []; }
    }

    const client = new maxt.Client(new EmptyAdapter());
    return {
      exchange: client.exchange.id,
      markets: (await client.markets(maxt.MarketKind.Spot)).length,
      supportsMarkets: client.supports(maxt.Feature.Markets),
    };
  });

  expect(result).toEqual({
    exchange: "binance",
    markets: 0,
    supportsMarkets: true,
  });
});
