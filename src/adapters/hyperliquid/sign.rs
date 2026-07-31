//! Hyperliquid action encoding and local wallet signing.
//!
//! Actions are encoded as named-key msgpack, hashed with the nonce and vault
//! flag, and signed as an EIP-712 `Agent` with secp256k1. The private key is used
//! only inside this module; requests contain the action, nonce, and signature.

use k256::ecdsa::SigningKey;
use serde::Serialize;
use sha3::{Digest, Keccak256};

use crate::error::{Error, Result};

use super::HyperliquidNetwork;
use super::parse::EXCHANGE;

/// Fixed EIP-712 chain id for L1 action signatures.
/// Network separation is encoded by the `Agent.source` field.
const AGENT_CHAIN_ID: u64 = 1337;

impl HyperliquidNetwork {
    /// EIP-712 `Agent.source` value for network separation.
    const fn agent_source(self) -> &'static str {
        match self {
            Self::Mainnet => "a",
            Self::Testnet => "b",
        }
    }
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

/// One wire order in an `order` action.
/// Field declaration order is preserved by named-key msgpack and affects the hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OrderWire {
    /// Asset id.
    pub(crate) a: u32,
    /// Whether this buys the base asset.
    pub(crate) b: bool,
    /// Limit price, as text.
    pub(crate) p: String,
    /// Size in the base asset, as text.
    pub(crate) s: String,
    /// Whether the order may only reduce a position.
    pub(crate) r: bool,
    /// Order type.
    pub(crate) t: OrderKind,
}

/// Limit-order payload stored under the wire field `t`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OrderKind {
    pub(crate) limit: LimitKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LimitKind {
    /// `Gtc`, `Ioc`, or `Alo`. Hyperliquid spells post-only `Alo`, for
    /// "add liquidity only".
    pub(crate) tif: &'static str,
}

/// An `order` action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OrderAction {
    #[serde(rename = "type")]
    pub(crate) action_type: &'static str,
    pub(crate) orders: Vec<OrderWire>,
    /// `na` for an ungrouped order.
    pub(crate) grouping: &'static str,
}

impl OrderAction {
    pub(crate) fn new(order: OrderWire) -> Self {
        Self {
            action_type: "order",
            orders: vec![order],
            grouping: "na",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CancelWire {
    /// Asset id.
    pub(crate) a: u32,
    /// The exchange's own order id.
    pub(crate) o: u64,
}

/// A `cancel` action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CancelAction {
    #[serde(rename = "type")]
    pub(crate) action_type: &'static str,
    pub(crate) cancels: Vec<CancelWire>,
}

impl CancelAction {
    pub(crate) fn new(asset: u32, order_id: u64) -> Self {
        Self {
            action_type: "cancel",
            cancels: vec![CancelWire {
                a: asset,
                o: order_id,
            }],
        }
    }
}

/// An `updateLeverage` action carrying leverage and margin mode together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LeverageAction {
    #[serde(rename = "type")]
    pub(crate) action_type: &'static str,
    pub(crate) asset: u32,
    #[serde(rename = "isCross")]
    pub(crate) is_cross: bool,
    pub(crate) leverage: u32,
}

impl LeverageAction {
    pub(crate) fn new(asset: u32, is_cross: bool, leverage: u32) -> Self {
        Self {
            action_type: "updateLeverage",
            asset,
            is_cross,
            leverage,
        }
    }
}

// ---------------------------------------------------------------------------
// Signing
// ---------------------------------------------------------------------------

/// A secp256k1 signature in the shape Hyperliquid expects on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Signature {
    pub(crate) r: String,
    pub(crate) s: String,
    /// Recovery id, offset by 27 the way Ethereum writes it.
    pub(crate) v: u8,
}

/// Parses a 32-byte hexadecimal secp256k1 private key.
/// The `0x` prefix is optional.
pub(crate) fn signing_key(private_key: &str) -> Result<SigningKey> {
    let text = private_key.trim();
    let text = text.strip_prefix("0x").unwrap_or(text);

    if text.len() != 64 || !text.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(Error::auth(
            "a Hyperliquid signing key is 32 bytes of hex, with or without a `0x` prefix",
        ));
    }
    let bytes =
        hex::decode(text).map_err(|err| Error::auth(format!("unreadable signing key: {err}")))?;

    SigningKey::from_slice(&bytes)
        .map_err(|err| Error::auth(format!("not a valid secp256k1 key: {err}")))
}

/// Derives the Ethereum address for a signing-key test vector.
#[cfg(test)]
pub(crate) fn address_of(key: &SigningKey) -> String {
    let public = key.verifying_key().to_encoded_point(false);
    let hash = Keccak256::digest(&public.as_bytes()[1..]);

    format!("0x{}", hex::encode(&hash[12..]))
}

/// Hashes named-key msgpack, an eight-byte big-endian nonce, and the vault flag.
fn action_hash(action_bytes: &[u8], nonce: u64) -> [u8; 32] {
    let mut data = Vec::with_capacity(action_bytes.len() + 9);
    data.extend_from_slice(action_bytes);
    data.extend_from_slice(&nonce.to_be_bytes());
    // Vault signing is not exposed, so the optional-vault flag is false.
    data.push(0);

    keccak(&data)
}

/// The EIP-712 digest of an `Agent` carrying an action hash.
fn agent_digest(action_hash: [u8; 32], source: &str) -> [u8; 32] {
    let mut agent = Vec::with_capacity(96);
    agent.extend_from_slice(&keccak(b"Agent(string source,bytes32 connectionId)"));
    agent.extend_from_slice(&keccak(source.as_bytes()));
    agent.extend_from_slice(&action_hash);
    let agent_hash = keccak(&agent);

    let mut chain_id = [0u8; 32];
    chain_id[24..].copy_from_slice(&AGENT_CHAIN_ID.to_be_bytes());

    let mut domain = Vec::with_capacity(160);
    domain.extend_from_slice(&keccak(
        b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
    ));
    domain.extend_from_slice(&keccak(b"Exchange"));
    domain.extend_from_slice(&keccak(b"1"));
    domain.extend_from_slice(&chain_id);
    // The verifying contract is the zero address.
    domain.extend_from_slice(&[0u8; 32]);
    let domain_separator = keccak(&domain);

    let mut digest = Vec::with_capacity(66);
    digest.extend_from_slice(b"\x19\x01");
    digest.extend_from_slice(&domain_separator);
    digest.extend_from_slice(&agent_hash);

    keccak(&digest)
}

fn keccak(input: &[u8]) -> [u8; 32] {
    Keccak256::digest(input).into()
}

/// Builds an `/exchange` body containing the action, millisecond nonce, and
/// EIP-712 signature.
pub(crate) fn signed_body<A: Serialize>(
    action: &A,
    private_key: &str,
    nonce: u64,
    network: HyperliquidNetwork,
) -> Result<String> {
    // Named-key msgpack is the signed representation.
    let action_bytes = rmp_serde::to_vec_named(action)
        .map_err(|err| Error::decode(format!("could not encode hyperliquid action: {err}")))?;
    let digest = agent_digest(action_hash(&action_bytes, nonce), network.agent_source());
    let signature = sign(private_key, digest)?;

    serde_json::to_string(&serde_json::json!({
        "action": action,
        "nonce": nonce,
        "signature": signature,
    }))
    .map_err(|err| Error::decode(format!("could not build hyperliquid request body: {err}")))
}

/// Signs a prepared digest.
fn sign(private_key: &str, digest: [u8; 32]) -> Result<Signature> {
    let key = signing_key(private_key)?;
    let (signature, recovery) = key
        .sign_prehash_recoverable(&digest)
        .map_err(|err| Error::auth(format!("could not sign the request: {err}")))?;

    Ok(Signature {
        // Indexing rather than `as_slice`, which `generic-array` deprecated
        // ahead of its 1.x release.
        r: scalar_hex(&signature.r().to_bytes()[..]),
        s: scalar_hex(&signature.s().to_bytes()[..]),
        v: recovery.to_byte() + 27,
    })
}

/// Formats a signature scalar as minimal `0x`-prefixed hexadecimal.
fn scalar_hex(bytes: &[u8]) -> String {
    let encoded = hex::encode(bytes);
    let trimmed = encoded.trim_start_matches('0');

    if trimmed.is_empty() {
        "0x0".to_string()
    } else {
        format!("0x{trimmed}")
    }
}

/// Validates the account-address shape and signing key, then lowercases the
/// address. The key is not required to derive the account address because an
/// approved API wallet may sign for another account.
pub(crate) fn check_wallet(address: &str, private_key: &str) -> Result<String> {
    let lowered = address.trim().to_ascii_lowercase();
    let is_address = lowered.len() == 42
        && lowered.starts_with("0x")
        && lowered[2..].chars().all(|digit| digit.is_ascii_hexdigit());
    if !is_address {
        return Err(Error::auth(format!(
            "`{address}` is not a 20-byte hex Hyperliquid account address"
        )));
    }
    signing_key(private_key)?;

    Ok(lowered)
}

/// Recovers an Ethereum address from a digest and signature in tests.
#[cfg(test)]
pub(crate) fn recover(digest: [u8; 32], signature: &Signature) -> Result<String> {
    use k256::ecdsa::{RecoveryId, VerifyingKey};

    let scalar = |text: &str, field: &'static str| -> Result<[u8; 32]> {
        let digits = text.strip_prefix("0x").unwrap_or(text);
        let padded = format!("{digits:0>64}");
        let bytes = hex::decode(&padded)
            .map_err(|err| Error::decode(format!("`{field}` is not hex: {err}")))?;
        let mut scalar = [0u8; 32];
        scalar.copy_from_slice(&bytes);
        Ok(scalar)
    };

    let mut serialized = [0u8; 64];
    serialized[..32].copy_from_slice(&scalar(&signature.r, "r")?);
    serialized[32..].copy_from_slice(&scalar(&signature.s, "s")?);

    let parsed = k256::ecdsa::Signature::from_slice(&serialized)
        .map_err(|err| Error::decode(format!("unreadable signature: {err}")))?;
    let recovery = RecoveryId::from_byte(signature.v - 27)
        .ok_or_else(|| Error::decode("signature recovery id is out of range"))?;
    let key = VerifyingKey::recover_from_prehash(&digest, &parsed, recovery)
        .map_err(|err| Error::decode(format!("could not recover the signer: {err}")))?;

    let public = key.to_encoded_point(false);
    Ok(format!(
        "0x{}",
        hex::encode(&Keccak256::digest(&public.as_bytes()[1..])[12..])
    ))
}

/// Builds the authentication error returned when no wallet is configured.
pub(crate) fn missing_wallet() -> Error {
    Error::auth(format!(
        "{EXCHANGE} signs private requests with a wallet; build the adapter with `with_wallet`"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Public SDK test vector. Never use this key outside tests.
    ///
    /// https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/exchange-endpoint
    const TEST_KEY: &str = "0x0123456789012345678901234567890123456789012345678901234567890123";
    const TEST_ADDRESS: &str = "0x14791697260e4c9a71f18484c9f997b308e59325";

    fn order() -> OrderAction {
        OrderAction::new(OrderWire {
            a: 0,
            b: true,
            p: "27123".to_string(),
            s: "1.2345".to_string(),
            r: false,
            t: OrderKind {
                limit: LimitKind { tif: "Gtc" },
            },
        })
    }

    #[test]
    fn the_documented_key_derives_the_documented_address() {
        let key = signing_key(TEST_KEY).expect("a documented key");

        assert_eq!(address_of(&key), TEST_ADDRESS);
        // The optional prefix does not affect the derived address.
        assert_eq!(
            address_of(&signing_key(&TEST_KEY[2..]).expect("the same key")),
            TEST_ADDRESS
        );
    }

    /// Order from the reference client's published action-hash vector.
    fn published_order() -> OrderAction {
        OrderAction::new(OrderWire {
            a: 4,
            b: true,
            p: "1670.1".to_string(),
            s: "0.0147".to_string(),
            r: false,
            t: OrderKind {
                limit: LimitKind { tif: "Ioc" },
            },
        })
    }

    /// Nonce used by the published action-hash vector.
    const PUBLISHED_NONCE: u64 = 1_677_777_606_040;

    /// Expected action hash for [`published_order`] at [`PUBLISHED_NONCE`].
    const PUBLISHED_ACTION_HASH: &str =
        "0fcbeda5ae3c4950a548021552a4fea2226858c4453571bf3f24ba017eac2908";

    #[test]
    fn the_published_action_hash_comes_out_byte_for_byte() {
        // Independent vector covers msgpack order, nonce bytes, and vault flag.
        let bytes = rmp_serde::to_vec_named(&published_order()).expect("an encodable action");

        assert_eq!(
            hex::encode(action_hash(&bytes, PUBLISHED_NONCE)),
            PUBLISHED_ACTION_HASH
        );
    }

    #[test]
    fn the_signed_body_signs_the_nonce_it_sends() {
        // Recover against the independent nonzero-nonce hash, not a hash
        // produced by the implementation under test.
        let mut published = [0u8; 32];
        published.copy_from_slice(&hex::decode(PUBLISHED_ACTION_HASH).expect("published hex"));

        for network in [HyperliquidNetwork::Mainnet, HyperliquidNetwork::Testnet] {
            let body = signed_body(&published_order(), TEST_KEY, PUBLISHED_NONCE, network)
                .expect("a signed body");
            let parsed: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
            let signature = Signature {
                r: parsed["signature"]["r"].as_str().expect("an r").to_string(),
                s: parsed["signature"]["s"].as_str().expect("an s").to_string(),
                v: parsed["signature"]["v"].as_u64().expect("a v") as u8,
            };

            assert_eq!(parsed["nonce"], PUBLISHED_NONCE);
            assert_eq!(
                recover(agent_digest(published, network.agent_source()), &signature)
                    .expect("a signer"),
                TEST_ADDRESS,
                "{network:?}"
            );
        }
    }

    #[test]
    fn the_published_signature_comes_out_scalar_for_scalar() {
        // Reference signatures cover action encoding, EIP-712 domain, network
        // source, and recovery-id formatting.
        let action = OrderAction::new(OrderWire {
            a: 1,
            b: true,
            p: "100".to_string(),
            s: "100".to_string(),
            r: false,
            t: OrderKind {
                limit: LimitKind { tif: "Gtc" },
            },
        });

        let signature = |network| -> serde_json::Value {
            let body = signed_body(&action, TEST_KEY, 0, network).expect("a signed body");
            serde_json::from_str::<serde_json::Value>(&body).expect("a JSON body")["signature"]
                .clone()
        };

        let mainnet = signature(HyperliquidNetwork::Mainnet);
        assert_eq!(
            mainnet["r"],
            "0xd65369825a9df5d80099e513cce430311d7d26ddf477f5b3a33d2806b100d78e"
        );
        assert_eq!(
            mainnet["s"],
            "0x2b54116ff64054968aa237c20ca9ff68000f977c93289157748a3162b6ea940e"
        );
        assert_eq!(mainnet["v"], 28);

        let testnet = signature(HyperliquidNetwork::Testnet);
        assert_eq!(
            testnet["r"],
            "0x82b2ba28e76b3d761093aaded1b1cdad4960b3af30212b343fb2e6cdfa4e3d54"
        );
        assert_eq!(
            testnet["s"],
            "0x6b53878fc99d26047f4d7e8c90eb98955a109f44209163f52d8dc4278cbbd9f5"
        );
        assert_eq!(testnet["v"], 27);
    }

    #[test]
    fn a_signature_recovers_to_the_wallet_that_made_it() {
        // This isolates signature recovery from the independent vectors above.
        let bytes = rmp_serde::to_vec_named(&order()).expect("an encodable action");
        let digest = agent_digest(action_hash(&bytes, 1_700_000_000_000), "a");
        let signature = sign(TEST_KEY, digest).expect("a signature");

        assert_eq!(recover(digest, &signature).expect("a signer"), TEST_ADDRESS);
    }

    #[test]
    fn mainnet_and_testnet_signatures_are_not_interchangeable() {
        // Different `source` values produce non-interchangeable digests.
        let bytes = rmp_serde::to_vec_named(&order()).expect("an encodable action");
        let hash = action_hash(&bytes, 1_700_000_000_000);
        let mainnet = agent_digest(hash, HyperliquidNetwork::Mainnet.agent_source());
        let testnet = agent_digest(hash, HyperliquidNetwork::Testnet.agent_source());

        assert_ne!(mainnet, testnet);

        let signed_for_testnet = sign(TEST_KEY, testnet).expect("a signature");
        assert_ne!(
            recover(mainnet, &signed_for_testnet).expect("some signer"),
            TEST_ADDRESS
        );
    }

    #[test]
    fn the_nonce_is_part_of_what_is_signed() {
        let bytes = rmp_serde::to_vec_named(&order()).expect("an encodable action");

        assert_ne!(action_hash(&bytes, 1), action_hash(&bytes, 2));
    }

    #[test]
    fn an_action_is_msgpacked_as_named_keys_in_declaration_order() {
        // Positional msgpack would encode arrays instead of named maps.
        let bytes = rmp_serde::to_vec_named(&order()).expect("an encodable action");
        let readable: serde_json::Value =
            rmp_serde::from_slice(&bytes).expect("a map, not an array");

        assert_eq!(readable["type"], "order");
        assert_eq!(readable["grouping"], "na");
        assert_eq!(readable["orders"][0]["a"], 0);
        assert_eq!(readable["orders"][0]["b"], true);
        assert_eq!(readable["orders"][0]["p"], "27123");
        assert_eq!(readable["orders"][0]["s"], "1.2345");
        assert_eq!(readable["orders"][0]["r"], false);
        assert_eq!(readable["orders"][0]["t"]["limit"]["tif"], "Gtc");
        // A three-field msgpack map starts with `0x83`.
        assert_eq!(bytes[0], 0x83);
    }

    #[test]
    fn the_signed_body_carries_the_action_the_signature_covers() {
        let body = signed_body(
            &order(),
            TEST_KEY,
            1_700_000_000_000,
            HyperliquidNetwork::Mainnet,
        )
        .expect("a signed body");
        let parsed: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");

        assert_eq!(parsed["nonce"], 1_700_000_000_000_u64);
        assert_eq!(parsed["action"]["type"], "order");
        assert_eq!(parsed["action"]["orders"][0]["p"], "27123");
        assert!(matches!(parsed["signature"]["v"].as_u64(), Some(27 | 28)));
        assert!(
            parsed["signature"]["r"]
                .as_str()
                .is_some_and(|r| r.starts_with("0x"))
        );
        // The request body contains no private-key material.
        assert!(!body.contains(TEST_KEY));
        assert!(!body.contains(&TEST_KEY[2..]));
    }

    #[test]
    fn signing_is_deterministic_so_a_retry_sends_the_identical_bytes() {
        // Identical inputs produce byte-identical signed requests.
        let first =
            signed_body(&order(), TEST_KEY, 42, HyperliquidNetwork::Mainnet).expect("a body");
        let second =
            signed_body(&order(), TEST_KEY, 42, HyperliquidNetwork::Mainnet).expect("a body");

        assert_eq!(first, second);
    }

    #[test]
    fn cancel_and_leverage_actions_use_hyperliquids_own_field_names() {
        let cancel: serde_json::Value =
            serde_json::to_value(CancelAction::new(3, 91_490_942)).expect("serializable");
        let leverage: serde_json::Value =
            serde_json::to_value(LeverageAction::new(3, false, 20)).expect("serializable");

        assert_eq!(cancel["type"], "cancel");
        assert_eq!(cancel["cancels"][0]["a"], 3);
        assert_eq!(cancel["cancels"][0]["o"], 91_490_942_u64);
        assert_eq!(leverage["type"], "updateLeverage");
        assert_eq!(leverage["isCross"], false);
        assert_eq!(leverage["leverage"], 20);
    }

    #[test]
    fn a_key_that_is_not_32_bytes_of_hex_is_refused_before_anything_is_signed() {
        for bad in [
            "",
            "0x",
            "not-hex",
            &TEST_KEY[..40],
            &format!("{TEST_KEY}00"),
        ] {
            assert!(matches!(signing_key(bad), Err(Error::Auth { .. })), "{bad}");
        }
    }

    #[test]
    fn an_address_that_is_not_an_account_is_refused() {
        assert!(check_wallet(TEST_ADDRESS, TEST_KEY).is_ok());
        assert!(matches!(
            check_wallet("0xabc", TEST_KEY),
            Err(Error::Auth { .. })
        ));
        assert!(matches!(
            check_wallet(TEST_ADDRESS, "not-a-key"),
            Err(Error::Auth { .. })
        ));
    }
}
