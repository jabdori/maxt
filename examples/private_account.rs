//! Reads balances and open orders without modifying the account.
//!
//! Credentials come from the environment, never from a file and never from the
//! source.
//!
//! ```text
//! export UPBIT_ACCESS_KEY=...
//! export UPBIT_SECRET_KEY=...
//! cargo run --example private_account
//! ```

use maxt::adapters::UpbitAdapter;
use maxt::{Client, Feature};

const ACCESS_KEY: &str = "UPBIT_ACCESS_KEY";
const SECRET_KEY: &str = "UPBIT_SECRET_KEY";

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let (Ok(access_key), Ok(secret_key)) = (std::env::var(ACCESS_KEY), std::env::var(SECRET_KEY))
    else {
        println!("set {ACCESS_KEY} and {SECRET_KEY} to a read-enabled Upbit key pair, then run");
        println!("this again. Nothing here places or cancels an order, so a key with trade");
        println!("permission is not needed.");
        return Ok(());
    };

    let client = Client::new(UpbitAdapter::new().with_credentials(access_key, secret_key));

    // Capability checks reflect the adapter's credential state.
    if !client.supports(Feature::Balances) {
        println!("{} cannot read balances as configured", client.exchange());
        return Ok(());
    }

    let balances = client.balances().await?;
    // Filter zero-total assets before printing.
    let funded: Vec<_> = balances.iter().filter(|b| !b.total().is_zero()).collect();
    println!(
        "{} holds {} funded assets:",
        client.exchange().display_name(),
        funded.len()
    );
    for balance in funded {
        println!(
            "  {:<8} {:>20} available, {:>20} locked",
            balance.asset, balance.available, balance.locked
        );
    }

    let orders = client.open_orders().await?;
    if orders.is_empty() {
        println!("\nno open orders");
        return Ok(());
    }

    println!("\n{} open orders:", orders.len());
    for order in &orders {
        // `None` means a market-sized order.
        let price = order
            .price
            .map_or_else(|| "market".to_string(), |price| price.to_string());
        println!(
            "  {} {:?} at {price}, {:?}, {} filled, {} left  (id {})",
            order.market,
            order.side,
            order.status,
            order.filled_quantity,
            order.remaining_quantity,
            order.id
        );
    }

    Ok(())
}
