//! Read an Upbit account when read-only credentials are present, and build
//! order and transfer requests without submitting a financial write.
//!
//! ```text
//! export UPBIT_ACCESS_KEY=...
//! export UPBIT_SECRET_KEY=...
//! cargo run --example account_and_safety
//! ```

use maxt::adapters::UpbitAdapter;
use maxt::{
    Client, Decimal, Exchange, Market, Network, OrderRequest, Side, Size, TransferHistoryRequest,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> maxt::Result<()> {
    let market = Market::spot(Exchange::Upbit, "BTC", "KRW");
    let draft = OrderRequest::limit(
        market,
        Side::Buy,
        Size::Base(Decimal::new(1, 4)),
        Decimal::from(100_000_u32),
    )
    .client_id("docs-example-only");
    let transfer_history = TransferHistoryRequest::new()
        .asset("BTC")
        .network(Network::Bitcoin)
        .limit(20);
    println!("order draft only; it was not sent: {draft:?}");
    println!("transfer-history request: {transfer_history:?}");

    let (Ok(access_key), Ok(secret_key)) = (
        std::env::var("UPBIT_ACCESS_KEY"),
        std::env::var("UPBIT_SECRET_KEY"),
    ) else {
        println!("set UPBIT_ACCESS_KEY and UPBIT_SECRET_KEY to run the read-only account section");
        return Ok(());
    };
    let client = Client::new(UpbitAdapter::new().with_credentials(access_key, secret_key));
    let balances = client.balances().await?;
    let orders = client.open_orders().await?;
    println!(
        "{} balances and {} open orders",
        balances.len(),
        orders.len()
    );
    Ok(())
}
