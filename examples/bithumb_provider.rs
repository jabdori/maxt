//! Read Bithumb-specific public market metadata.
//!
//! ```text
//! cargo run --example bithumb_provider
//! ```

use maxt::adapters::BithumbAdapter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> maxt::Result<()> {
    let adapter = BithumbAdapter::new();
    let warnings = adapter.market_warnings().await?;
    let notices = adapter.notices(Some(5)).await?;
    let fees = adapter.transfer_fees("BTC").await?;
    println!(
        "{} warning rows, {} notices, {} BTC fee rows",
        warnings.len(),
        notices.len(),
        fees.len()
    );
    Ok(())
}
