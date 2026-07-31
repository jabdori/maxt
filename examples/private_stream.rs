//! Prints live balance and order updates without placing or cancelling orders.
//!
//! Credentials come from the environment, never from a file and never from the
//! source.
//!
//! ```text
//! export UPBIT_ACCESS_KEY=...
//! export UPBIT_SECRET_KEY=...
//! cargo run --example private_stream
//! ```
//!
//! This credentialed path is not exercised by automated live tests.
//!
//! It exits after 20 events or 60 seconds, whichever comes first. An idle
//! account prints nothing and still exits cleanly.

use std::time::{Duration, Instant};

use futures_util::StreamExt;
use maxt::adapters::UpbitAdapter;
use maxt::{AccountEvent, Client, Feature};

const ACCESS_KEY: &str = "UPBIT_ACCESS_KEY";
const SECRET_KEY: &str = "UPBIT_SECRET_KEY";
const EVENT_LIMIT: usize = 20;
const TIME_LIMIT: Duration = Duration::from_secs(60);

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let (Ok(access_key), Ok(secret_key)) = (std::env::var(ACCESS_KEY), std::env::var(SECRET_KEY))
    else {
        println!("set {ACCESS_KEY} and {SECRET_KEY} to a read-enabled Upbit key pair, then run");
        println!("this again.");
        return Ok(());
    };

    let client = Client::new(UpbitAdapter::new().with_credentials(access_key, secret_key));

    if !client.supports(Feature::AccountStream) {
        println!(
            "{} does not support {} as configured",
            client.exchange(),
            Feature::AccountStream
        );
        return Ok(());
    }

    let mut stream = client.subscribe_account().await?;
    println!("watching the account for {EVENT_LIMIT} events or {TIME_LIMIT:?}");

    let started = Instant::now();
    let mut seen = 0usize;

    while seen < EVENT_LIMIT {
        let Some(left) = TIME_LIMIT.checked_sub(started.elapsed()) else {
            break;
        };
        let Ok(item) = tokio::time::timeout(left, stream.next()).await else {
            println!("no event before the {TIME_LIMIT:?} deadline. An idle account, not a failure");
            break;
        };
        // Only `None` ends the stream; report `Err` items and keep polling.
        let Some(event) = item else {
            break;
        };
        let event = match event {
            Ok(event) => event,
            Err(error) => {
                println!("-- {error}");
                continue;
            }
        };

        seen += 1;
        match event {
            AccountEvent::Balance(balance) => println!(
                "balance {:<8} {} available, {} locked",
                balance.asset, balance.available, balance.locked
            ),
            AccountEvent::Order(order) => println!(
                "order   {} {:?} {:?}, {} filled, {} left  (id {})",
                order.market,
                order.side,
                order.status,
                order.filled_quantity,
                order.remaining_quantity,
                order.id
            ),
            // Reconnects may miss fills; re-read balances and orders over REST.
            AccountEvent::Reconnected => {
                println!("-- reconnected; re-read balances and open orders to resynchronize")
            }
            other => println!("-- {other:?}"),
        }
    }

    println!("done after {seen} events in {:?}", started.elapsed());
    Ok(())
}
