//! Safe composition of deposit-address and withdrawal operations.

use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{
    Adapter, AssetNetwork, ChainDestination, ChainTransferRequest, Client, DepositAddressRequest,
    Error, Exchange, ExchangeDestination, ExchangeTransferRequest, Network, Result, Timestamp,
    TransferDestination, TransferErrorKind, TravelRuleRequirement, WithdrawRequest, Withdrawal,
    WithdrawalQuote,
};

const DEFAULT_PLAN_LIFETIME_NANOS: i64 = 60_000_000_000;

/// One asset on one exchange, optionally pinned to a network.
#[derive(Debug)]
pub struct Wallet<'a, A> {
    client: &'a Client<A>,
    asset: String,
    network: Option<Network>,
}

impl<A: Adapter> Client<A> {
    /// Selects an asset wallet, with `None` allowing one unambiguous network.
    pub fn wallet(&self, asset: impl Into<String>, network: Option<Network>) -> Wallet<'_, A> {
        Wallet {
            client: self,
            asset: asset.into().to_ascii_uppercase(),
            network,
        }
    }
}

impl<'a, A: Adapter> Wallet<'a, A> {
    /// Asset symbol, uppercase.
    pub fn asset(&self) -> &str {
        &self.asset
    }

    /// Explicit network, or `None` when unique-network selection is allowed.
    pub fn network(&self) -> Option<&Network> {
        self.network.as_ref()
    }

    /// Checks both exchanges and prepares a withdrawal without submitting it.
    pub async fn prepare_transfer_to<B: Adapter>(
        &self,
        destination: &Wallet<'_, B>,
        amount: Decimal,
    ) -> Result<PreparedTransfer<'a, A>> {
        if self.asset != destination.asset {
            return Err(Error::transfer(
                TransferErrorKind::AssetMismatch,
                format!(
                    "source asset {} differs from destination asset {}",
                    self.asset, destination.asset,
                ),
            ));
        }
        let request = ExchangeTransferRequest {
            asset: self.asset.clone(),
            source_network: self.network.clone(),
            destination_network: destination.network.clone(),
            amount,
        };
        let plan = prepare_exchange_transfer(
            self.client.adapter(),
            destination.client.adapter(),
            &request,
        )
        .await?;
        Ok(PreparedTransfer {
            source: self.client,
            plan,
        })
    }

    /// Prepares a withdrawal to an explicit on-chain destination.
    ///
    /// This path never discovers or guesses a bridge address. It is intended
    /// for destinations such as a Hyperliquid account address supplied by the
    /// caller for a network the source exchange directly supports.
    pub async fn prepare_transfer_to_chain(
        &self,
        destination: ChainDestination,
        amount: Decimal,
    ) -> Result<PreparedTransfer<'a, A>> {
        let request = ChainTransferRequest {
            asset: self.asset.clone(),
            source_network: self.network.clone(),
            destination,
            amount,
        };
        let plan = prepare_chain_transfer(self.client.adapter(), &request).await?;
        Ok(PreparedTransfer {
            source: self.client,
            plan,
        })
    }
}

/// Checks both exchange adapters and returns a detached transfer plan.
///
/// No withdrawal is submitted. The destination address, live network state,
/// source limits, fee, allowlist, and Travel Rule requirement are checked.
pub async fn prepare_exchange_transfer(
    source: &dyn Adapter,
    destination: &dyn Adapter,
    request: &ExchangeTransferRequest,
) -> Result<TransferPlan> {
    let asset = request.asset.to_ascii_uppercase();
    if asset.is_empty() {
        return Err(Error::invalid_request("asset", "asset must not be empty"));
    }
    if request.amount <= Decimal::ZERO {
        return Err(Error::transfer(
            TransferErrorKind::AmountOutOfRange,
            "amount must be greater than zero",
        ));
    }
    if let (Some(source), Some(destination)) =
        (&request.source_network, &request.destination_network)
        && !source.same_chain(destination)
    {
        return Err(Error::transfer(
            TransferErrorKind::NetworkMismatch,
            format!("source network {source} differs from destination network {destination}"),
        ));
    }

    let source_networks = source.asset_networks(&asset).await?;
    let destination_networks = destination.asset_networks(&asset).await?;
    let (source_network, destination_network) = select_networks(
        &source_networks,
        request.source_network.as_ref(),
        &destination_networks,
        request.destination_network.as_ref(),
    )?;
    validate_amount(
        request.amount,
        source_network.minimum_withdrawal,
        source_network.maximum_withdrawal,
    )?;

    let address = destination
        .deposit_address(&DepositAddressRequest::new(
            &asset,
            destination_network.network.clone(),
        ))
        .await?;
    if address.exchange != destination.exchange()
        || !address.asset.eq_ignore_ascii_case(&asset)
        || !address.network.same_chain(&source_network.network)
    {
        return Err(Error::adapter(format!(
            "{} returned a deposit destination that does not match {asset} on {}",
            destination.exchange(),
            source_network.network,
        )));
    }
    if destination_network.memo_required && address.memo.is_none() {
        return Err(Error::transfer(
            TransferErrorKind::MemoRequired,
            format!(
                "{} {asset} deposits on {} require a memo or tag",
                destination.exchange(),
                destination_network.network,
            ),
        ));
    }
    let destination_address = address.address.ok_or_else(|| {
        Error::transfer(
            TransferErrorKind::DestinationUnavailable,
            format!(
                "{} has not issued a {asset} deposit address on {} yet",
                destination.exchange(),
                destination_network.network,
            ),
        )
    })?;

    let withdrawal = WithdrawRequest::new(
        &asset,
        source_network.network.clone(),
        request.amount,
        TransferDestination::Exchange(ExchangeDestination {
            exchange: address.exchange,
            asset: address.asset,
            network: address.network,
            address: destination_address,
            memo: address.memo,
        }),
    )
    .client_id(Uuid::new_v4().to_string());
    let quote = source.prepare_withdrawal(&withdrawal).await?;
    validate_quote(request.amount, &quote)?;

    Ok(transfer_plan(
        source.exchange(),
        Some(destination.exchange()),
        withdrawal,
        quote,
    ))
}

/// Checks a source adapter and returns a plan for an explicit chain address.
///
/// No bridge or deposit address is discovered or guessed.
pub async fn prepare_chain_transfer(
    source: &dyn Adapter,
    request: &ChainTransferRequest,
) -> Result<TransferPlan> {
    let asset = request.asset.to_ascii_uppercase();
    if asset.is_empty() {
        return Err(Error::invalid_request("asset", "asset must not be empty"));
    }
    if asset != request.destination.asset.to_ascii_uppercase() {
        return Err(Error::transfer(
            TransferErrorKind::AssetMismatch,
            format!(
                "source asset {asset} differs from destination asset {}",
                request.destination.asset,
            ),
        ));
    }
    if let Some(network) = &request.source_network
        && !network.same_chain(&request.destination.network)
    {
        return Err(Error::transfer(
            TransferErrorKind::NetworkMismatch,
            format!(
                "source network {network} differs from destination network {}",
                request.destination.network,
            ),
        ));
    }
    if request.amount <= Decimal::ZERO {
        return Err(Error::transfer(
            TransferErrorKind::AmountOutOfRange,
            "amount must be greater than zero",
        ));
    }

    let networks = source.asset_networks(&asset).await?;
    let source_network = networks
        .iter()
        .find(|candidate| {
            candidate.withdrawal_enabled
                && candidate.network.same_chain(&request.destination.network)
        })
        .ok_or_else(|| {
            Error::transfer(
                TransferErrorKind::NetworkUnavailable,
                format!(
                    "{} is not enabled for {asset} withdrawal",
                    request.destination.network,
                ),
            )
        })?;
    validate_amount(
        request.amount,
        source_network.minimum_withdrawal,
        source_network.maximum_withdrawal,
    )?;
    let withdrawal = WithdrawRequest::new(
        &asset,
        source_network.network.clone(),
        request.amount,
        TransferDestination::Chain(request.destination.clone()),
    )
    .client_id(Uuid::new_v4().to_string());
    let quote = source.prepare_withdrawal(&withdrawal).await?;
    validate_quote(request.amount, &quote)?;
    Ok(transfer_plan(source.exchange(), None, withdrawal, quote))
}

/// Data checked before a cross-exchange withdrawal is submitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferPlan {
    /// Source exchange.
    pub source: Exchange,
    /// Destination exchange.
    pub destination: Option<Exchange>,
    /// Exact withdrawal that will be submitted.
    pub request: WithdrawRequest,
    /// Live source-exchange checks.
    pub quote: WithdrawalQuote,
    /// Local creation time.
    pub created_at: Timestamp,
    /// Time after which execution is rejected locally.
    pub expires_at: Timestamp,
}

/// A checked transfer plan tied to its source client.
#[derive(Debug)]
pub struct PreparedTransfer<'a, A> {
    source: &'a Client<A>,
    plan: TransferPlan,
}

impl<A: Adapter> PreparedTransfer<'_, A> {
    /// Checked plan details for review or persistence.
    pub fn plan(&self) -> &TransferPlan {
        &self.plan
    }

    /// Detaches the checked data without submitting a withdrawal.
    pub fn into_plan(self) -> TransferPlan {
        self.plan
    }

    /// Submits the checked withdrawal once.
    ///
    /// This returns source-exchange acknowledgement, not destination deposit
    /// completion. An indeterminate transport error is never retried here.
    pub async fn execute(self) -> Result<Withdrawal> {
        execute_transfer_plan(self.source.adapter(), &self.plan).await
    }
}

/// Executes one checked transfer plan exactly once.
///
/// Success means the source exchange accepted the withdrawal. It does not mean
/// that the destination exchange credited a deposit. Transport failures are
/// returned without retrying the financial write.
pub async fn execute_transfer_plan(
    source: &dyn Adapter,
    plan: &TransferPlan,
) -> Result<Withdrawal> {
    if source.exchange() != plan.source {
        return Err(Error::invalid_request(
            "plan.source",
            format!("plan belongs to {}, not {}", plan.source, source.exchange()),
        ));
    }
    if Timestamp::now() >= plan.expires_at {
        return Err(Error::transfer(
            TransferErrorKind::PlanExpired,
            format!("plan expired at {}", plan.expires_at),
        ));
    }
    source.withdraw(&plan.request).await
}

fn transfer_plan(
    source: Exchange,
    destination: Option<Exchange>,
    request: WithdrawRequest,
    quote: WithdrawalQuote,
) -> TransferPlan {
    let created_at = Timestamp::now();
    let expires_at = quote.expires_at.unwrap_or_else(|| {
        Timestamp::from_nanos(
            created_at
                .as_nanos()
                .saturating_add(DEFAULT_PLAN_LIFETIME_NANOS),
        )
    });
    TransferPlan {
        source,
        destination,
        request,
        quote,
        created_at,
        expires_at,
    }
}

fn select_networks<'a>(
    source: &'a [AssetNetwork],
    requested_source: Option<&Network>,
    destination: &'a [AssetNetwork],
    requested_destination: Option<&Network>,
) -> Result<(&'a AssetNetwork, &'a AssetNetwork)> {
    let requested = requested_source.or(requested_destination);
    if let Some(network) = requested {
        let source = source.iter().find(|candidate| {
            candidate.withdrawal_enabled && candidate.network.same_chain(network)
        });
        let destination = destination
            .iter()
            .find(|candidate| candidate.deposit_enabled && candidate.network.same_chain(network));
        return match (source, destination) {
            (Some(source), Some(destination)) => Ok((source, destination)),
            _ => Err(Error::transfer(
                TransferErrorKind::NetworkUnavailable,
                format!("{network} is not enabled for both withdrawal and deposit"),
            )),
        };
    }

    let mut matches = Vec::new();
    for source_network in source.iter().filter(|value| value.withdrawal_enabled) {
        if let Some(destination_network) = destination.iter().find(|value| {
            value.deposit_enabled && value.network.same_chain(&source_network.network)
        }) && !matches
            .iter()
            .any(|(existing, _): &(&AssetNetwork, &AssetNetwork)| {
                existing.network.same_chain(&source_network.network)
            })
        {
            matches.push((source_network, destination_network));
        }
    }
    match matches.len() {
        0 => Err(Error::transfer(
            TransferErrorKind::NetworkMismatch,
            "source and destination have no enabled network in common",
        )),
        1 => Ok(matches[0]),
        _ => Err(Error::transfer(
            TransferErrorKind::AmbiguousNetwork,
            "more than one enabled network is shared; select one explicitly",
        )),
    }
}

fn validate_amount(
    amount: Decimal,
    minimum: Option<Decimal>,
    maximum: Option<Decimal>,
) -> Result<()> {
    if minimum.is_some_and(|minimum| amount < minimum) {
        return Err(Error::transfer(
            TransferErrorKind::AmountOutOfRange,
            format!(
                "amount {amount} is below minimum {}",
                minimum.unwrap_or_default()
            ),
        ));
    }
    if maximum.is_some_and(|maximum| amount > maximum) {
        return Err(Error::transfer(
            TransferErrorKind::AmountOutOfRange,
            format!(
                "amount {amount} exceeds maximum {}",
                maximum.unwrap_or_default()
            ),
        ));
    }
    Ok(())
}

fn validate_quote(amount: Decimal, quote: &WithdrawalQuote) -> Result<()> {
    validate_amount(amount, quote.minimum_amount, quote.maximum_amount)?;
    if quote.address_allowed == Some(false) {
        return Err(Error::transfer(
            TransferErrorKind::AddressNotAllowed,
            "destination address is not allowed by the source account",
        ));
    }
    if matches!(quote.travel_rule, TravelRuleRequirement::Required { .. }) {
        return Err(Error::transfer(
            TransferErrorKind::TravelRuleRequired,
            "provider-specific Travel Rule data or consent is required",
        ));
    }
    if quote.fee.is_some_and(|fee| fee >= amount) {
        return Err(Error::transfer(
            TransferErrorKind::AmountOutOfRange,
            "withdrawal fee must be smaller than the amount",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::{
        BoxFuture, DepositAddress, Feature, Page, TransferHistoryRequest, WithdrawalFee,
        WithdrawalStatus,
    };

    #[derive(Clone)]
    struct MockWalletAdapter {
        exchange: Exchange,
        networks: Vec<AssetNetwork>,
        address: Option<DepositAddress>,
        quote: WithdrawalQuote,
        withdrawals: Arc<AtomicUsize>,
    }

    impl Adapter for MockWalletAdapter {
        fn exchange(&self) -> Exchange {
            self.exchange
        }

        fn supports(&self, feature: Feature) -> bool {
            matches!(
                feature,
                Feature::AssetNetworks
                    | Feature::DepositAddresses
                    | Feature::WithdrawalQuotes
                    | Feature::Withdrawals
            )
        }

        fn asset_networks(&self, _asset: &str) -> BoxFuture<'_, Result<Vec<AssetNetwork>>> {
            Box::pin(async { Ok(self.networks.clone()) })
        }

        fn deposit_address(
            &self,
            _request: &DepositAddressRequest,
        ) -> BoxFuture<'_, Result<DepositAddress>> {
            Box::pin(async {
                self.address
                    .clone()
                    .ok_or_else(|| Error::adapter("mock has no deposit address"))
            })
        }

        fn prepare_withdrawal(
            &self,
            _request: &WithdrawRequest,
        ) -> BoxFuture<'_, Result<WithdrawalQuote>> {
            Box::pin(async { Ok(self.quote.clone()) })
        }

        fn withdraw(&self, request: &WithdrawRequest) -> BoxFuture<'_, Result<Withdrawal>> {
            self.withdrawals.fetch_add(1, Ordering::SeqCst);
            let request = request.clone();
            Box::pin(async move {
                Ok(Withdrawal {
                    id: "withdrawal-1".to_string(),
                    asset: request.asset,
                    network: Some(request.network),
                    provider_network: Some("ARBITRUM".to_string()),
                    amount: request.amount,
                    fee: Some(Decimal::ONE),
                    destination: Some(request.destination),
                    status: WithdrawalStatus::Pending,
                    provider_status: "pending".to_string(),
                    tx_id: None,
                    created_at: Some(Timestamp::now()),
                })
            })
        }

        fn deposits(
            &self,
            _request: &TransferHistoryRequest,
        ) -> BoxFuture<'_, Result<Page<crate::Deposit>>> {
            Box::pin(async {
                Ok(Page {
                    items: vec![],
                    next: None,
                })
            })
        }
    }

    fn rule(exchange: Exchange, network: Network) -> AssetNetwork {
        AssetNetwork {
            exchange,
            asset: "ETH".to_string(),
            provider_id: network.id().to_string(),
            network,
            deposit_enabled: true,
            withdrawal_enabled: true,
            withdrawal_fee: Some(WithdrawalFee::Fixed(Decimal::ONE)),
            minimum_withdrawal: Some(Decimal::ONE),
            maximum_withdrawal: Some(Decimal::from(100)),
            memo_required: false,
        }
    }

    fn adapter(
        exchange: Exchange,
        networks: Vec<AssetNetwork>,
        address_network: Network,
    ) -> MockWalletAdapter {
        MockWalletAdapter {
            exchange,
            networks,
            address: Some(DepositAddress {
                exchange,
                asset: "ETH".to_string(),
                network: address_network,
                address: Some("0x0000000000000000000000000000000000000001".to_string()),
                memo: None,
            }),
            quote: WithdrawalQuote {
                fee: Some(Decimal::ONE),
                expected_receive: Some(Decimal::from(9)),
                minimum_amount: Some(Decimal::ONE),
                maximum_amount: Some(Decimal::from(100)),
                address_allowed: Some(true),
                travel_rule: TravelRuleRequirement::NotRequired,
                expires_at: None,
            },
            withdrawals: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[tokio::test]
    async fn one_shared_network_prepares_and_executes_once() {
        let source_adapter = adapter(
            Exchange::Binance,
            vec![rule(Exchange::Binance, Network::Arbitrum)],
            Network::Arbitrum,
        );
        let calls = source_adapter.withdrawals.clone();
        let destination_adapter = adapter(
            Exchange::Upbit,
            vec![rule(Exchange::Upbit, Network::Arbitrum)],
            Network::Arbitrum,
        );
        let source = Client::new(source_adapter);
        let destination = Client::new(destination_adapter);

        let prepared = source
            .wallet("ETH", None)
            .prepare_transfer_to(&destination.wallet("ETH", None), Decimal::TEN)
            .await
            .unwrap();
        assert_eq!(prepared.plan().request.network, Network::Arbitrum);

        let withdrawal = prepared.execute().await.unwrap();
        assert_eq!(withdrawal.status, WithdrawalStatus::Pending);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn explicit_chain_destination_does_not_require_a_deposit_address() {
        let source_adapter = adapter(
            Exchange::Binance,
            vec![rule(Exchange::Binance, Network::Arbitrum)],
            Network::Ethereum,
        );
        let calls = source_adapter.withdrawals.clone();
        let source = Client::new(source_adapter);

        let prepared = source
            .wallet("ETH", Some(Network::Arbitrum))
            .prepare_transfer_to_chain(
                ChainDestination {
                    asset: "ETH".to_string(),
                    network: Network::Arbitrum,
                    address: "0x0000000000000000000000000000000000000002".to_string(),
                    memo: None,
                },
                Decimal::TEN,
            )
            .await
            .unwrap();

        assert_eq!(prepared.plan().destination, None);
        assert!(matches!(
            prepared.plan().request.destination,
            TransferDestination::Chain(_)
        ));
        prepared.execute().await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn different_explicit_chains_fail_before_any_withdrawal() {
        let source_adapter = adapter(
            Exchange::Binance,
            vec![rule(Exchange::Binance, Network::Ethereum)],
            Network::Ethereum,
        );
        let calls = source_adapter.withdrawals.clone();
        let destination_adapter = adapter(
            Exchange::Upbit,
            vec![rule(Exchange::Upbit, Network::Arbitrum)],
            Network::Arbitrum,
        );
        let source = Client::new(source_adapter);
        let destination = Client::new(destination_adapter);

        let result = source
            .wallet("ETH", Some(Network::Ethereum))
            .prepare_transfer_to(
                &destination.wallet("ETH", Some(Network::Arbitrum)),
                Decimal::TEN,
            )
            .await;

        assert!(matches!(
            result,
            Err(Error::Transfer {
                kind: TransferErrorKind::NetworkMismatch,
                ..
            })
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn automatic_selection_rejects_more_than_one_shared_chain() {
        let source = Client::new(adapter(
            Exchange::Binance,
            vec![
                rule(Exchange::Binance, Network::Ethereum),
                rule(Exchange::Binance, Network::Arbitrum),
            ],
            Network::Ethereum,
        ));
        let destination = Client::new(adapter(
            Exchange::Upbit,
            vec![
                rule(Exchange::Upbit, Network::Ethereum),
                rule(Exchange::Upbit, Network::Arbitrum),
            ],
            Network::Ethereum,
        ));

        let result = source
            .wallet("ETH", None)
            .prepare_transfer_to(&destination.wallet("ETH", None), Decimal::TEN)
            .await;

        assert!(matches!(
            result,
            Err(Error::Transfer {
                kind: TransferErrorKind::AmbiguousNetwork,
                ..
            })
        ));
    }
}
