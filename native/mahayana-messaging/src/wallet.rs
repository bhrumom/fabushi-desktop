use crate::actor::ActorId;
use crate::payment::Money;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use thiserror::Error;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};
use zeroize::Zeroize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WalletAccountId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletAccount {
    pub id: WalletAccountId,
    pub owner_id: ActorId,
    pub balances_minor: BTreeMap<String, i64>,
    pub frozen: bool,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl WalletAccount {
    pub fn balance(&self, currency: &str) -> i64 {
        self.balances_minor
            .get(&normalize_currency(currency))
            .copied()
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LedgerEntryKind {
    Credit,
    Transfer,
    Refund,
    Adjustment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerEntry {
    pub id: String,
    pub request_id: String,
    pub kind: LedgerEntryKind,
    pub from_account_id: Option<WalletAccountId>,
    pub to_account_id: Option<WalletAccountId>,
    pub amount: Money,
    pub reference: Option<String>,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletLedger {
    pub accounts: BTreeMap<WalletAccountId, WalletAccount>,
    pub entries: BTreeMap<String, LedgerEntry>,
    request_entries: BTreeMap<String, String>,
    #[serde(default)]
    pub runtime: WalletRuntimeState,
}

impl WalletLedger {
    pub fn create_account(
        &mut self,
        id: WalletAccountId,
        owner_id: ActorId,
        now_ms: i64,
    ) -> Result<(), WalletError> {
        if id.0.trim().is_empty() {
            return Err(WalletError::InvalidAccountId);
        }
        if self.accounts.contains_key(&id) {
            return Err(WalletError::DuplicateAccount(id));
        }
        self.accounts.insert(
            id.clone(),
            WalletAccount {
                id,
                owner_id,
                balances_minor: BTreeMap::new(),
                frozen: false,
                created_at_ms: now_ms,
                updated_at_ms: now_ms,
            },
        );
        Ok(())
    }

    pub fn credit(
        &mut self,
        request_id: impl Into<String>,
        account_id: &WalletAccountId,
        amount: Money,
        reference: Option<String>,
        now_ms: i64,
    ) -> Result<LedgerEntry, WalletError> {
        let request_id = request_id.into();
        validate_amount(&amount)?;
        let currency = normalize_currency(&amount.currency);
        if let Some(existing) = self.entry_for_request(&request_id) {
            return same_request(
                existing,
                LedgerEntryKind::Credit,
                None,
                Some(account_id),
                &currency,
                amount.amount_minor,
                reference.as_deref(),
                &request_id,
            );
        }
        let account = self
            .accounts
            .get_mut(account_id)
            .ok_or_else(|| WalletError::AccountNotFound(account_id.clone()))?;
        if account.frozen {
            return Err(WalletError::AccountFrozen(account_id.clone()));
        }
        let balance = account.balances_minor.entry(currency.clone()).or_default();
        *balance = balance
            .checked_add(amount.amount_minor)
            .ok_or(WalletError::BalanceOverflow)?;
        account.updated_at_ms = now_ms;
        let entry = LedgerEntry {
            id: format!("ledger:{}", self.entries.len().saturating_add(1)),
            request_id: request_id.clone(),
            kind: LedgerEntryKind::Credit,
            from_account_id: None,
            to_account_id: Some(account_id.clone()),
            amount: Money {
                currency,
                amount_minor: amount.amount_minor,
            },
            reference,
            created_at_ms: now_ms,
        };
        self.insert_entry(entry.clone());
        Ok(entry)
    }

    pub fn transfer(
        &mut self,
        request_id: impl Into<String>,
        from_account_id: &WalletAccountId,
        to_account_id: &WalletAccountId,
        amount: Money,
        reference: Option<String>,
        now_ms: i64,
    ) -> Result<LedgerEntry, WalletError> {
        let request_id = request_id.into();
        validate_amount(&amount)?;
        if from_account_id == to_account_id {
            return Err(WalletError::SameAccountTransfer);
        }
        let currency = normalize_currency(&amount.currency);
        if let Some(existing) = self.entry_for_request(&request_id) {
            return same_request(
                existing,
                LedgerEntryKind::Transfer,
                Some(from_account_id),
                Some(to_account_id),
                &currency,
                amount.amount_minor,
                reference.as_deref(),
                &request_id,
            );
        }
        let debit_balance = self
            .accounts
            .get(from_account_id)
            .ok_or_else(|| WalletError::AccountNotFound(from_account_id.clone()))?
            .balance(&currency);
        if self
            .accounts
            .get(from_account_id)
            .is_some_and(|account| account.frozen)
        {
            return Err(WalletError::AccountFrozen(from_account_id.clone()));
        }
        if self
            .accounts
            .get(to_account_id)
            .ok_or_else(|| WalletError::AccountNotFound(to_account_id.clone()))?
            .frozen
        {
            return Err(WalletError::AccountFrozen(to_account_id.clone()));
        }
        if debit_balance < amount.amount_minor {
            return Err(WalletError::InsufficientFunds {
                account_id: from_account_id.clone(),
                currency,
                available_minor: debit_balance,
                required_minor: amount.amount_minor,
            });
        }

        {
            let from = self
                .accounts
                .get_mut(from_account_id)
                .ok_or_else(|| WalletError::AccountNotFound(from_account_id.clone()))?;
            let balance = from.balances_minor.entry(currency.clone()).or_default();
            *balance = balance
                .checked_sub(amount.amount_minor)
                .ok_or(WalletError::BalanceOverflow)?;
            from.updated_at_ms = now_ms;
        }
        {
            let to = self
                .accounts
                .get_mut(to_account_id)
                .ok_or_else(|| WalletError::AccountNotFound(to_account_id.clone()))?;
            let balance = to.balances_minor.entry(currency.clone()).or_default();
            *balance = balance
                .checked_add(amount.amount_minor)
                .ok_or(WalletError::BalanceOverflow)?;
            to.updated_at_ms = now_ms;
        }

        let entry = LedgerEntry {
            id: format!("ledger:{}", self.entries.len().saturating_add(1)),
            request_id: request_id.clone(),
            kind: LedgerEntryKind::Transfer,
            from_account_id: Some(from_account_id.clone()),
            to_account_id: Some(to_account_id.clone()),
            amount: Money {
                currency,
                amount_minor: amount.amount_minor,
            },
            reference,
            created_at_ms: now_ms,
        };
        self.insert_entry(entry.clone());
        Ok(entry)
    }

    pub fn refund_transfer(
        &mut self,
        request_id: impl Into<String>,
        original_entry_id: &str,
        now_ms: i64,
    ) -> Result<LedgerEntry, WalletError> {
        let request_id = request_id.into();
        let expected_reference = format!("refund:{original_entry_id}");
        if let Some(existing) = self.entry_for_request(&request_id) {
            let amount = &existing.amount;
            return same_request(
                existing,
                LedgerEntryKind::Refund,
                existing.from_account_id.as_ref(),
                existing.to_account_id.as_ref(),
                &normalize_currency(&amount.currency),
                amount.amount_minor,
                Some(&expected_reference),
                &request_id,
            )
            .and_then(|entry| {
                let matches_original_direction = self
                    .entries
                    .get(original_entry_id)
                    .is_some_and(|original| {
                        original.kind == LedgerEntryKind::Transfer
                            && entry.from_account_id == original.to_account_id
                            && entry.to_account_id == original.from_account_id
                            && entry.amount == original.amount
                    });
                matches_original_direction
                    .then_some(entry)
                    .ok_or_else(|| WalletError::RequestConflict(request_id.clone()))
            });
        }
        let original = self
            .entries
            .get(original_entry_id)
            .cloned()
            .ok_or_else(|| WalletError::EntryNotFound(original_entry_id.to_string()))?;
        if original.kind != LedgerEntryKind::Transfer {
            return Err(WalletError::NotRefundable(original_entry_id.to_string()));
        }
        let original_from = original
            .from_account_id
            .clone()
            .ok_or_else(|| WalletError::NotRefundable(original_entry_id.to_string()))?;
        let original_to = original
            .to_account_id
            .clone()
            .ok_or_else(|| WalletError::NotRefundable(original_entry_id.to_string()))?;
        let mut entry = self.transfer(
            request_id.clone(),
            &original_to,
            &original_from,
            original.amount.clone(),
            Some(expected_reference),
            now_ms,
        )?;
        entry.kind = LedgerEntryKind::Refund;
        self.entries.insert(entry.id.clone(), entry.clone());
        Ok(entry)
    }

    pub fn freeze(
        &mut self,
        account_id: &WalletAccountId,
        frozen: bool,
    ) -> Result<(), WalletError> {
        let account = self
            .accounts
            .get_mut(account_id)
            .ok_or_else(|| WalletError::AccountNotFound(account_id.clone()))?;
        account.frozen = frozen;
        Ok(())
    }

    pub fn entry_for_request(&self, request_id: &str) -> Option<&LedgerEntry> {
        self.request_entries
            .get(request_id)
            .and_then(|entry_id| self.entries.get(entry_id))
    }

    fn insert_entry(&mut self, entry: LedgerEntry) {
        self.request_entries
            .insert(entry.request_id.clone(), entry.id.clone());
        self.entries.insert(entry.id.clone(), entry);
    }
}

fn same_request(
    existing: &LedgerEntry,
    kind: LedgerEntryKind,
    from_account_id: Option<&WalletAccountId>,
    to_account_id: Option<&WalletAccountId>,
    currency: &str,
    amount_minor: i64,
    reference: Option<&str>,
    request_id: &str,
) -> Result<LedgerEntry, WalletError> {
    let matches = existing.kind == kind
        && existing.from_account_id.as_ref() == from_account_id
        && existing.to_account_id.as_ref() == to_account_id
        && existing.amount.currency == currency
        && existing.amount.amount_minor == amount_minor
        && existing.reference.as_deref() == reference;
    if matches {
        Ok(existing.clone())
    } else {
        Err(WalletError::RequestConflict(request_id.to_string()))
    }
}

fn validate_amount(amount: &Money) -> Result<(), WalletError> {
    if amount.currency.trim().len() < 3 || amount.amount_minor <= 0 {
        return Err(WalletError::InvalidAmount);
    }
    Ok(())
}

fn normalize_currency(currency: &str) -> String {
    currency.trim().to_ascii_uppercase()
}


pub const WALLET_RATE_REFRESH_MS: i64 = 5 * 60 * 1_000;
pub const WALLET_RATE_RETRY_MS: i64 = 30 * 1_000;
pub const WALLET_RECOVERY_SEED_BYTES: usize = 215;
const WALLET_KEY_BYTES: usize = 32;
const WALLET_KEY_NONCE_BYTES: usize = 24;
const WALLET_RECOVERY_PUBLIC_KEY_BYTES: usize = 32;
const WALLET_RECOVERY_NONCE_BYTES: usize = 24;
const WALLET_KEY_ALGORITHM: &str = "XChaCha20Poly1305";
const WALLET_KEY_AAD_PREFIX: &[u8] = b"fabushi-wallet-key-v1";
const WALLET_RECOVERY_AAD_PREFIX: &[u8] = b"fabushi-wallet-recovery-share-v1";

/// Secret-bearing bytes never participate in serde and are cleansed on drop.
/// The explicit wrapper makes it difficult to accidentally persist or log
/// wallet/recovery key material while allowing platform adapters to hand a
/// short-lived wrapping key to the canonical wallet owner.
pub struct WalletSecretBytes(Vec<u8>);

impl WalletSecretBytes {
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self(bytes.into())
    }

    pub fn expose(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for WalletSecretBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("WalletSecretBytes([REDACTED])")
    }
}

impl Drop for WalletSecretBytes {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WalletKeyProtectionKind {
    Passcode,
    System,
    Hardware,
    External,
}

#[derive(Debug)]
pub struct WalletProtectionMaterial {
    /// Opaque platform-owned handle. The canonical wallet never persists the
    /// wrapping key itself, only this handle plus authenticated ciphertext.
    pub reference: String,
    pub wrapping_key: WalletSecretBytes,
}

pub trait WalletProtectionProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn kind(&self) -> WalletKeyProtectionKind;

    /// The provider returns a wrapping key and opaque reference. It is never
    /// handed the wallet key, so device/passcode protection cannot become a
    /// second wallet-secret owner.
    fn enroll(&self) -> Result<WalletProtectionMaterial, WalletProtectionError>;
    fn unwrap(&self, reference: &str) -> Result<WalletSecretBytes, WalletProtectionError>;
    fn remove(&self, reference: &str) -> Result<(), WalletProtectionError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtectedWalletKey {
    pub version: u32,
    pub algorithm: String,
    pub provider_id: String,
    pub protection_kind: WalletKeyProtectionKind,
    pub provider_reference: String,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletKeyProtectionState {
    pub epoch: u64,
    pub locked: bool,
    pub envelope: Option<ProtectedWalletKey>,
}

impl Default for WalletKeyProtectionState {
    fn default() -> Self {
        Self {
            epoch: 0,
            locked: true,
            envelope: None,
        }
    }
}

impl WalletKeyProtectionState {
    pub fn install(
        &mut self,
        expected_epoch: u64,
        provider: &dyn WalletProtectionProvider,
        wallet_key: &WalletSecretBytes,
    ) -> Result<u64, WalletProtectionError> {
        self.require_epoch(expected_epoch)?;
        let envelope = protect_wallet_key(provider, wallet_key)?;
        self.epoch = self
            .epoch
            .checked_add(1)
            .ok_or(WalletProtectionError::EpochOverflow)?;
        self.envelope = Some(envelope);
        self.locked = true;
        Ok(self.epoch)
    }

    pub fn unlock(
        &mut self,
        expected_epoch: u64,
        provider: &dyn WalletProtectionProvider,
    ) -> Result<WalletSecretBytes, WalletProtectionError> {
        self.require_epoch(expected_epoch)?;
        let envelope = self
            .envelope
            .as_ref()
            .ok_or(WalletProtectionError::Absent)?;
        let result = unprotect_wallet_key(provider, envelope)?;
        self.locked = false;
        Ok(result)
    }

    /// A replacement is committed before the old platform handle is retired.
    /// If retirement fails, the new encrypted envelope remains authoritative
    /// and the caller receives a typed retirement failure rather than rolling
    /// security state back to an old factor.
    pub fn rotate(
        &mut self,
        expected_epoch: u64,
        current_provider: &dyn WalletProtectionProvider,
        replacement_provider: &dyn WalletProtectionProvider,
    ) -> Result<u64, WalletProtectionError> {
        self.require_epoch(expected_epoch)?;
        let previous = self
            .envelope
            .clone()
            .ok_or(WalletProtectionError::Absent)?;
        let wallet_key = unprotect_wallet_key(current_provider, &previous)?;
        let replacement = protect_wallet_key(replacement_provider, &wallet_key)?;
        self.epoch = self
            .epoch
            .checked_add(1)
            .ok_or(WalletProtectionError::EpochOverflow)?;
        self.envelope = Some(replacement);
        self.locked = true;
        current_provider
            .remove(&previous.provider_reference)
            .map_err(|_| WalletProtectionError::RetirementFailed)?;
        Ok(self.epoch)
    }

    pub fn remove(
        &mut self,
        expected_epoch: u64,
        provider: &dyn WalletProtectionProvider,
    ) -> Result<u64, WalletProtectionError> {
        self.require_epoch(expected_epoch)?;
        let previous = self
            .envelope
            .clone()
            .ok_or(WalletProtectionError::Absent)?;
        require_matching_provider(provider, &previous)?;
        provider.remove(&previous.provider_reference)?;
        self.epoch = self
            .epoch
            .checked_add(1)
            .ok_or(WalletProtectionError::EpochOverflow)?;
        self.envelope = None;
        self.locked = true;
        Ok(self.epoch)
    }

    fn require_epoch(&self, expected_epoch: u64) -> Result<(), WalletProtectionError> {
        if self.epoch == expected_epoch {
            Ok(())
        } else {
            Err(WalletProtectionError::StaleEpoch {
                expected: expected_epoch,
                current: self.epoch,
            })
        }
    }
}

fn wallet_key_aad(
    provider_id: &str,
    kind: WalletKeyProtectionKind,
    reference: &str,
) -> Vec<u8> {
    let mut aad = Vec::with_capacity(
        WALLET_KEY_AAD_PREFIX.len() + provider_id.len() + reference.len() + 16,
    );
    aad.extend_from_slice(WALLET_KEY_AAD_PREFIX);
    aad.push(0);
    aad.extend_from_slice(provider_id.as_bytes());
    aad.push(0);
    aad.extend_from_slice(format!("{kind:?}").as_bytes());
    aad.push(0);
    aad.extend_from_slice(reference.as_bytes());
    aad
}

fn require_wrapping_key(key: &WalletSecretBytes) -> Result<(), WalletProtectionError> {
    if key.expose().len() == WALLET_KEY_BYTES {
        Ok(())
    } else {
        Err(WalletProtectionError::InvalidWrappingKey)
    }
}

fn require_matching_provider(
    provider: &dyn WalletProtectionProvider,
    envelope: &ProtectedWalletKey,
) -> Result<(), WalletProtectionError> {
    if envelope.version != 1 || envelope.algorithm != WALLET_KEY_ALGORITHM {
        return Err(WalletProtectionError::UnsupportedEnvelope);
    }
    if envelope.provider_id != provider.id() || envelope.protection_kind != provider.kind() {
        return Err(WalletProtectionError::WrongProvider);
    }
    Ok(())
}

pub fn protect_wallet_key(
    provider: &dyn WalletProtectionProvider,
    wallet_key: &WalletSecretBytes,
) -> Result<ProtectedWalletKey, WalletProtectionError> {
    if wallet_key.expose().len() != WALLET_KEY_BYTES {
        return Err(WalletProtectionError::InvalidWalletKey);
    }
    let material = provider.enroll()?;
    if material.reference.trim().is_empty() {
        return Err(WalletProtectionError::InvalidProviderReference);
    }
    require_wrapping_key(&material.wrapping_key)?;
    let mut nonce = [0u8; WALLET_KEY_NONCE_BYTES];
    getrandom::getrandom(&mut nonce).map_err(|_| WalletProtectionError::EntropyUnavailable)?;
    let aad = wallet_key_aad(provider.id(), provider.kind(), &material.reference);
    let cipher = XChaCha20Poly1305::new_from_slice(material.wrapping_key.expose())
        .map_err(|_| WalletProtectionError::InvalidWrappingKey)?;
    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: wallet_key.expose(),
                aad: &aad,
            },
        )
        .map_err(|_| WalletProtectionError::EncryptionFailed)?;
    Ok(ProtectedWalletKey {
        version: 1,
        algorithm: WALLET_KEY_ALGORITHM.to_string(),
        provider_id: provider.id().to_string(),
        protection_kind: provider.kind(),
        provider_reference: material.reference,
        nonce: nonce.to_vec(),
        ciphertext,
    })
}

pub fn unprotect_wallet_key(
    provider: &dyn WalletProtectionProvider,
    envelope: &ProtectedWalletKey,
) -> Result<WalletSecretBytes, WalletProtectionError> {
    require_matching_provider(provider, envelope)?;
    if envelope.nonce.len() != WALLET_KEY_NONCE_BYTES {
        return Err(WalletProtectionError::CorruptEnvelope);
    }
    let material = provider.unwrap(&envelope.provider_reference)?;
    require_wrapping_key(&material)?;
    let aad = wallet_key_aad(
        &envelope.provider_id,
        envelope.protection_kind,
        &envelope.provider_reference,
    );
    let cipher = XChaCha20Poly1305::new_from_slice(material.expose())
        .map_err(|_| WalletProtectionError::InvalidWrappingKey)?;
    let plaintext = cipher
        .decrypt(
            XNonce::from_slice(&envelope.nonce),
            Payload {
                msg: &envelope.ciphertext,
                aad: &aad,
            },
        )
        .map_err(|_| WalletProtectionError::AuthenticationFailed)?;
    if plaintext.len() != WALLET_KEY_BYTES {
        return Err(WalletProtectionError::CorruptEnvelope);
    }
    Ok(WalletSecretBytes::new(plaintext))
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WalletProtectionError {
    #[error("wallet key protection was cancelled")]
    Cancelled,
    #[error("wallet key protection is unavailable")]
    Unavailable,
    #[error("wallet key protection material is absent")]
    Absent,
    #[error("wallet key protection provider rejected authentication")]
    AuthenticationFailed,
    #[error("wallet key protection provider does not match the persisted envelope")]
    WrongProvider,
    #[error("wallet key protection envelope is unsupported")]
    UnsupportedEnvelope,
    #[error("wallet key protection envelope is corrupt")]
    CorruptEnvelope,
    #[error("wallet key must contain exactly 32 bytes")]
    InvalidWalletKey,
    #[error("wallet wrapping key must contain exactly 32 bytes")]
    InvalidWrappingKey,
    #[error("wallet protection provider returned an invalid reference")]
    InvalidProviderReference,
    #[error("secure entropy is unavailable")]
    EntropyUnavailable,
    #[error("wallet key encryption failed")]
    EncryptionFailed,
    #[error("wallet protection epoch is stale: expected {expected}, current {current}")]
    StaleEpoch { expected: u64, current: u64 },
    #[error("wallet protection epoch overflowed")]
    EpochOverflow,
    #[error("the replacement key protection committed but the old handle could not be retired")]
    RetirementFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletRateState {
    pub selected_currency: String,
    /// Source-neutral quote in millionths of selected fiat per one wallet
    /// asset. Integer storage avoids binary floating-point persistence drift.
    pub quote_micros: BTreeMap<String, i64>,
    pub last_updated_at_ms: Option<i64>,
    pub refresh_at_ms: i64,
    pub retrying: bool,
}

impl Default for WalletRateState {
    fn default() -> Self {
        Self {
            selected_currency: "USD".to_string(),
            quote_micros: BTreeMap::new(),
            last_updated_at_ms: None,
            refresh_at_ms: 0,
            retrying: false,
        }
    }
}

impl WalletRateState {
    pub fn set_currency(&mut self, currency: &str) -> Result<(), WalletRateError> {
        let normalized = checked_currency(currency)?;
        self.selected_currency = normalized;
        Ok(())
    }

    pub fn apply_snapshot(
        &mut self,
        rates: impl IntoIterator<Item = (String, i64)>,
        now_ms: i64,
    ) -> Result<(), WalletRateError> {
        let mut parsed = BTreeMap::new();
        for (currency, quote_micros) in rates {
            let currency = checked_currency(&currency)?;
            if quote_micros <= 0 {
                return Err(WalletRateError::InvalidQuote(currency));
            }
            parsed.insert(currency, quote_micros);
        }
        if parsed.is_empty() {
            return Err(WalletRateError::EmptySnapshot);
        }
        self.quote_micros = parsed;
        self.last_updated_at_ms = Some(now_ms);
        self.refresh_at_ms = now_ms.saturating_add(WALLET_RATE_REFRESH_MS);
        self.retrying = false;
        Ok(())
    }

    pub fn mark_refresh_failure(&mut self, now_ms: i64) {
        self.refresh_at_ms = now_ms.saturating_add(WALLET_RATE_RETRY_MS);
        self.retrying = true;
    }

    pub fn current_quote_micros(&self) -> Option<i64> {
        self.quote_micros.get(&self.selected_currency).copied()
    }

    pub fn currencies(&self) -> Vec<String> {
        let mut currencies = self.quote_micros.keys().cloned().collect::<BTreeSet<_>>();
        currencies.insert(self.selected_currency.clone());
        currencies.into_iter().collect()
    }

    pub fn refresh_due(&self, now_ms: i64) -> bool {
        now_ms >= self.refresh_at_ms
    }
}

fn checked_currency(currency: &str) -> Result<String, WalletRateError> {
    let normalized = normalize_currency(currency);
    if normalized.len() == 3 && normalized.bytes().all(|byte| byte.is_ascii_uppercase()) {
        Ok(normalized)
    } else {
        Err(WalletRateError::InvalidCurrency(currency.to_string()))
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WalletRateError {
    #[error("wallet fiat currency is invalid: {0}")]
    InvalidCurrency(String),
    #[error("wallet fiat quote is invalid for {0}")]
    InvalidQuote(String),
    #[error("wallet fiat-rate snapshot is empty")]
    EmptySnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnrampProviderInfo {
    pub id: String,
    /// None means the provider did not advertise or the optional discovery
    /// step failed; callers must still be able to create a session without a
    /// base currency.
    pub base_currencies: Option<BTreeSet<String>>,
}

impl OnrampProviderInfo {
    pub fn new(
        id: impl Into<String>,
        base_currencies: Option<impl IntoIterator<Item = String>>,
    ) -> Self {
        Self {
            id: id.into(),
            base_currencies: base_currencies.map(|currencies| {
                currencies
                    .into_iter()
                    .filter_map(|currency| checked_currency(&currency).ok())
                    .collect()
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WalletOnrampStatus {
    ResolvingProvider,
    CreatingSession,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletOnrampRequest {
    pub request_id: u64,
    pub address: String,
    pub asset: String,
    pub base_currency: Option<String>,
    pub provider_id: Option<String>,
    pub status: WalletOnrampStatus,
    pub session_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnrampSessionSpec {
    pub request_id: u64,
    pub address: String,
    pub asset: String,
    pub provider_id: String,
    pub base_currency: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletOnrampState {
    pub next_request_id: u64,
    pub active: Option<WalletOnrampRequest>,
}

impl WalletOnrampState {
    /// Beginning a new request invalidates the previous request generation.
    /// Late provider/session callbacks therefore cannot settle a replacement.
    pub fn begin(
        &mut self,
        address: impl Into<String>,
        asset: impl Into<String>,
        base_currency: Option<&str>,
    ) -> Result<u64, WalletOnrampError> {
        let address = address.into().trim().to_string();
        let asset = asset.into().trim().to_ascii_uppercase();
        if address.is_empty() || asset.is_empty() {
            return Err(WalletOnrampError::InvalidRequest);
        }
        let base_currency = base_currency
            .filter(|value| !value.trim().is_empty())
            .map(checked_currency)
            .transpose()
            .map_err(|_| WalletOnrampError::InvalidRequest)?;
        self.next_request_id = self
            .next_request_id
            .checked_add(1)
            .ok_or(WalletOnrampError::RequestIdOverflow)?;
        let request_id = self.next_request_id;
        self.active = Some(WalletOnrampRequest {
            request_id,
            address,
            asset,
            base_currency,
            provider_id: None,
            status: WalletOnrampStatus::ResolvingProvider,
            session_url: None,
        });
        Ok(request_id)
    }

    pub fn resolve_provider(
        &mut self,
        request_id: u64,
        providers: &[OnrampProviderInfo],
    ) -> Result<OnrampSessionSpec, WalletOnrampError> {
        let request = self.current_mut(request_id)?;
        if request.status != WalletOnrampStatus::ResolvingProvider {
            return Err(WalletOnrampError::InvalidState);
        }
        let provider = providers
            .iter()
            .find(|provider| !provider.id.trim().is_empty())
            .ok_or(WalletOnrampError::ProviderUnavailable)?;
        let base_currency = request.base_currency.as_ref().and_then(|selected| {
            provider
                .base_currencies
                .as_ref()
                .filter(|currencies| currencies.contains(selected))
                .map(|_| selected.clone())
        });
        request.provider_id = Some(provider.id.clone());
        request.status = WalletOnrampStatus::CreatingSession;
        Ok(OnrampSessionSpec {
            request_id,
            address: request.address.clone(),
            asset: request.asset.clone(),
            provider_id: provider.id.clone(),
            base_currency,
        })
    }

    pub fn complete(
        &mut self,
        request_id: u64,
        session_url: Option<&str>,
    ) -> Result<Option<String>, WalletOnrampError> {
        let request = self.current_mut(request_id)?;
        if request.status != WalletOnrampStatus::CreatingSession {
            return Err(WalletOnrampError::InvalidState);
        }
        let url = session_url
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        request.status = if url.is_some() {
            WalletOnrampStatus::Completed
        } else {
            WalletOnrampStatus::Failed
        };
        request.session_url = url.clone();
        Ok(url)
    }

    pub fn cancel(&mut self, request_id: u64) -> Result<(), WalletOnrampError> {
        let request = self.current_mut(request_id)?;
        if matches!(
            request.status,
            WalletOnrampStatus::Completed | WalletOnrampStatus::Failed
        ) {
            return Err(WalletOnrampError::InvalidState);
        }
        request.status = WalletOnrampStatus::Cancelled;
        Ok(())
    }

    fn current_mut(
        &mut self,
        request_id: u64,
    ) -> Result<&mut WalletOnrampRequest, WalletOnrampError> {
        let request = self
            .active
            .as_mut()
            .ok_or(WalletOnrampError::NoActiveRequest)?;
        if request.request_id != request_id {
            return Err(WalletOnrampError::StaleRequest {
                expected: request.request_id,
                received: request_id,
            });
        }
        Ok(request)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WalletOnrampError {
    #[error("wallet funding request is invalid")]
    InvalidRequest,
    #[error("wallet funding provider is unavailable")]
    ProviderUnavailable,
    #[error("wallet funding request has no active operation")]
    NoActiveRequest,
    #[error("wallet funding request is stale: expected {expected}, received {received}")]
    StaleRequest { expected: u64, received: u64 },
    #[error("wallet funding request is in an invalid state")]
    InvalidState,
    #[error("wallet funding request id overflowed")]
    RequestIdOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletExistingBalanceRequest {
    pub serial: u64,
    pub panel_generation: u64,
    pub network_generation: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletExistingBalanceState {
    pub requested_generation: Option<u64>,
    pub in_flight: Option<WalletExistingBalanceRequest>,
    pub next_serial: u64,
    pub url: Option<String>,
}

impl WalletExistingBalanceState {
    fn clear_opening(&mut self) {
        self.requested_generation = None;
        self.in_flight = None;
        self.url = None;
    }

    fn begin(
        &mut self,
        panel_generation: u64,
        network_generation: u64,
    ) -> Result<Option<WalletExistingBalanceRequest>, WalletExistingBalanceError> {
        if panel_generation == 0 {
            return Err(WalletExistingBalanceError::PanelNotOpen);
        }
        if network_generation == 0 {
            return Err(WalletExistingBalanceError::NetworkNotReady);
        }
        if self.requested_generation == Some(panel_generation) {
            return Ok(None);
        }
        self.next_serial = self
            .next_serial
            .checked_add(1)
            .ok_or(WalletExistingBalanceError::SerialOverflow)?;
        let request = WalletExistingBalanceRequest {
            serial: self.next_serial,
            panel_generation,
            network_generation,
        };
        self.requested_generation = Some(panel_generation);
        self.in_flight = Some(request);
        Ok(Some(request))
    }

    fn apply(
        &mut self,
        panel_generation: u64,
        current_network_generation: u64,
        serial: u64,
        url: Option<String>,
    ) -> Result<bool, WalletExistingBalanceError> {
        let request = self.take_matching(panel_generation, serial)?;
        if request.network_generation != current_network_generation {
            return Ok(false);
        }
        self.url = normalize_existing_balance_url(url)?;
        Ok(true)
    }

    fn fail(
        &mut self,
        panel_generation: u64,
        serial: u64,
    ) -> Result<(), WalletExistingBalanceError> {
        self.take_matching(panel_generation, serial)?;
        Ok(())
    }

    fn take_matching(
        &mut self,
        panel_generation: u64,
        serial: u64,
    ) -> Result<WalletExistingBalanceRequest, WalletExistingBalanceError> {
        if serial == 0 {
            return Err(WalletExistingBalanceError::InvalidSerial);
        }
        let request = self
            .in_flight
            .ok_or(WalletExistingBalanceError::NoActiveRequest)?;
        if request.serial != serial || request.panel_generation != panel_generation {
            return Err(WalletExistingBalanceError::StaleRequest {
                expected: self.in_flight,
                received_serial: serial,
                received_panel_generation: panel_generation,
            });
        }
        self.in_flight = None;
        Ok(request)
    }
}

fn normalize_existing_balance_url(
    url: Option<String>,
) -> Result<Option<String>, WalletExistingBalanceError> {
    let Some(url) = url else {
        return Ok(None);
    };
    let trimmed = url.trim();
    if trimmed.is_empty()
        || trimmed.len() > 4096
        || trimmed.chars().any(char::is_control)
    {
        return Err(WalletExistingBalanceError::InvalidUrl);
    }
    Ok(Some(trimmed.to_string()))
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WalletExistingBalanceError {
    #[error("wallet panel is not open")]
    PanelNotOpen,
    #[error("wallet network generation is not ready")]
    NetworkNotReady,
    #[error("wallet existing-balance request serial is invalid")]
    InvalidSerial,
    #[error("wallet existing-balance request serial overflowed")]
    SerialOverflow,
    #[error("wallet existing-balance request is not active")]
    NoActiveRequest,
    #[error("wallet existing-balance request is stale")]
    StaleRequest {
        expected: Option<WalletExistingBalanceRequest>,
        received_serial: u64,
        received_panel_generation: u64,
    },
    #[error("wallet existing-balance url is invalid")]
    InvalidUrl,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletPanelState {
    pub generation: u64,
    pub visible: bool,
    pub minimized: bool,
    pub active: bool,
    pub transactions_visible: bool,
    /// Existing legacy-balance discovery belongs to one visible panel opening.
    /// It is live service state and must never survive restart.
    #[serde(skip, default)]
    pub existing_balance: WalletExistingBalanceState,
}

impl WalletPanelState {
    /// The surface is singleton state: show restores/minimizes the current
    /// generation instead of constructing a parallel wallet window.
    pub fn show(&mut self) -> u64 {
        if !self.visible {
            self.generation = self.generation.saturating_add(1);
            self.existing_balance.clear_opening();
        }
        self.visible = true;
        self.minimized = false;
        self.active = true;
        self.generation
    }

    pub fn minimize(&mut self) -> bool {
        if !self.visible {
            return false;
        }
        self.minimized = true;
        self.active = false;
        true
    }

    pub fn close(&mut self) -> bool {
        if !self.visible {
            return false;
        }
        self.visible = false;
        self.minimized = false;
        self.active = false;
        self.existing_balance.clear_opening();
        true
    }

    pub fn begin_existing_balance_request(
        &mut self,
        network_generation: u64,
    ) -> Result<Option<WalletExistingBalanceRequest>, WalletExistingBalanceError> {
        if !self.visible {
            return Err(WalletExistingBalanceError::PanelNotOpen);
        }
        self.existing_balance.begin(self.generation, network_generation)
    }

    pub fn apply_existing_balance_result(
        &mut self,
        current_network_generation: u64,
        serial: u64,
        url: Option<String>,
    ) -> Result<bool, WalletExistingBalanceError> {
        if !self.visible {
            return Err(WalletExistingBalanceError::PanelNotOpen);
        }
        self.existing_balance.apply(
            self.generation,
            current_network_generation,
            serial,
            url,
        )
    }

    pub fn fail_existing_balance_request(
        &mut self,
        serial: u64,
    ) -> Result<(), WalletExistingBalanceError> {
        if !self.visible {
            return Err(WalletExistingBalanceError::PanelNotOpen);
        }
        self.existing_balance.fail(self.generation, serial)
    }

    pub fn set_transactions_visible(&mut self, visible: bool) {
        self.transactions_visible = visible;
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WalletEmulationActionKind {
    Withdraw,
    Deposit,
    Transfer,
    Excess,
    CallContract,
    DeployContract,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WalletEmulationActionSide {
    None,
    Incoming,
    Outgoing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletEmulationAction {
    pub kind: WalletEmulationActionKind,
    pub side: WalletEmulationActionSide,
    pub amount_nano: Option<i64>,
    pub counterparty: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WalletEmulationStatus {
    Shown,
    Failed,
    Aborted,
    Incomplete,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletEmulation {
    pub actions: Vec<WalletEmulationAction>,
    pub net_nano: i64,
    pub status: WalletEmulationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletEmulationSourceAction {
    pub kind: String,
    pub succeeded: bool,
    pub details_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletEmulationSource {
    pub trace_succeeded: bool,
    pub is_incomplete: bool,
    pub actions: Vec<WalletEmulationSourceAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WalletEmulationSourceKind {
    Transfer,
    CallContract,
    ContractDeploy,
    Skipped,
    Other,
}

fn wallet_emulation_source_kind(kind: &str) -> WalletEmulationSourceKind {
    match kind {
        "ton_transfer" | "extra_currency_transfer" => WalletEmulationSourceKind::Transfer,
        "call_contract" => WalletEmulationSourceKind::CallContract,
        "contract_deploy" => WalletEmulationSourceKind::ContractDeploy,
        "unknown" => WalletEmulationSourceKind::Skipped,
        _ => WalletEmulationSourceKind::Other,
    }
}

fn wallet_emulation_details(action: &WalletEmulationSourceAction) -> serde_json::Map<String, serde_json::Value> {
    serde_json::from_str::<serde_json::Value>(&action.details_json)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default()
}

fn wallet_emulation_address(
    details: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> String {
    details
        .get(key)
        .and_then(serde_json::Value::as_str)
        .filter(|value| validate_wallet_address(value).is_ok())
        .unwrap_or_default()
        .to_string()
}

fn wallet_emulation_nano(
    details: &serde_json::Map<String, serde_json::Value>,
) -> Option<i64> {
    let text = details.get("value")?.as_str()?;
    if text.is_empty() || !text.as_bytes().iter().all(u8::is_ascii_digit) {
        return None;
    }
    text.parse().ok()
}

fn wallet_emulation_opcode(
    details: &serde_json::Map<String, serde_json::Value>,
) -> Option<u32> {
    let value = details.get("opcode")?;
    if let Some(text) = value.as_str() {
        let digits = text
            .strip_prefix("0x")
            .or_else(|| text.strip_prefix("0X"))?;
        if digits.is_empty()
            || digits.len() > 8
            || !digits.as_bytes().iter().all(u8::is_ascii_hexdigit)
        {
            return None;
        }
        return u32::from_str_radix(digits, 16).ok();
    }
    if let Some(number) = value.as_i64() {
        if number < i32::MIN as i64 || number > u32::MAX as i64 {
            return None;
        }
        return Some(if number < 0 {
            (number as i32) as u32
        } else {
            number as u32
        });
    }
    value
        .as_u64()
        .filter(|number| *number <= u32::MAX as u64)
        .map(|number| number as u32)
}

fn wallet_emulation_side(
    source: &str,
    destination: &str,
    own: &str,
) -> WalletEmulationActionSide {
    if !source.is_empty() && source == own {
        WalletEmulationActionSide::Outgoing
    } else if !destination.is_empty() && destination == own {
        WalletEmulationActionSide::Incoming
    } else {
        WalletEmulationActionSide::None
    }
}

fn wallet_emulation_action_kind(
    kind: WalletEmulationSourceKind,
    side: WalletEmulationActionSide,
    details: &serde_json::Map<String, serde_json::Value>,
) -> WalletEmulationActionKind {
    match kind {
        WalletEmulationSourceKind::Transfer => match side {
            WalletEmulationActionSide::Outgoing => WalletEmulationActionKind::Withdraw,
            WalletEmulationActionSide::Incoming => WalletEmulationActionKind::Deposit,
            WalletEmulationActionSide::None => WalletEmulationActionKind::Transfer,
        },
        WalletEmulationSourceKind::CallContract => {
            if wallet_emulation_opcode(details) == Some(0xd53276db) {
                WalletEmulationActionKind::Excess
            } else {
                WalletEmulationActionKind::CallContract
            }
        }
        WalletEmulationSourceKind::ContractDeploy => WalletEmulationActionKind::DeployContract,
        WalletEmulationSourceKind::Skipped | WalletEmulationSourceKind::Other => {
            WalletEmulationActionKind::Unknown
        }
    }
}

pub fn parse_wallet_emulation(
    source: &WalletEmulationSource,
    wallet_address: &str,
    request_total_nano: i64,
) -> WalletEmulation {
    let mut status = if !source.trace_succeeded {
        let failed = source.actions.iter().any(|action| {
            !action.succeeded
                && wallet_emulation_source_kind(&action.kind) != WalletEmulationSourceKind::Skipped
        });
        if failed {
            WalletEmulationStatus::Failed
        } else {
            WalletEmulationStatus::Aborted
        }
    } else if source.is_incomplete {
        WalletEmulationStatus::Incomplete
    } else {
        WalletEmulationStatus::Shown
    };

    let own = if validate_wallet_address(wallet_address).is_ok() {
        wallet_address
    } else {
        ""
    };
    let mut valid = !own.is_empty();
    let mut credited = 0i64;
    let mut actions = Vec::new();

    if valid {
        for source_action in &source.actions {
            let source_kind = wallet_emulation_source_kind(&source_action.kind);
            if source_kind == WalletEmulationSourceKind::Skipped {
                continue;
            }
            let details = wallet_emulation_details(source_action);
            let source_address = wallet_emulation_address(&details, "source");
            let destination = wallet_emulation_address(&details, "destination");
            if source_kind == WalletEmulationSourceKind::ContractDeploy {
                actions.push(WalletEmulationAction {
                    kind: WalletEmulationActionKind::DeployContract,
                    side: WalletEmulationActionSide::None,
                    amount_nano: None,
                    counterparty: destination,
                });
                continue;
            }
            let side = wallet_emulation_side(&source_address, &destination, own);
            let amount_nano = wallet_emulation_nano(&details);
            let basic = matches!(
                source_kind,
                WalletEmulationSourceKind::Transfer | WalletEmulationSourceKind::CallContract
            );
            if basic && destination == own {
                if let Some(amount) = amount_nano {
                    match credited.checked_add(amount) {
                        Some(next) => credited = next,
                        None => valid = false,
                    }
                }
            }
            let counterparty = if side == WalletEmulationActionSide::Incoming {
                source_address
            } else {
                destination
            };
            actions.push(WalletEmulationAction {
                kind: wallet_emulation_action_kind(source_kind, side, &details),
                side,
                amount_nano,
                counterparty,
            });
        }
    }

    let net_nano = if valid {
        match credited.checked_sub(request_total_nano) {
            Some(value) => value,
            None => {
                valid = false;
                0
            }
        }
    } else {
        0
    };
    if status == WalletEmulationStatus::Shown && (!valid || actions.is_empty()) {
        status = WalletEmulationStatus::Empty;
    }
    WalletEmulation {
        actions,
        net_nano,
        status,
    }
}

pub const OUTBOUND_TRANSFER_MAX_RECORDS: usize = 64;
pub const OUTBOUND_TRANSFER_MAX_BYTES: usize = 4 * 1024 * 1024;
pub const OUTBOUND_TRANSFER_TOKEN_MAX_BYTES: usize = 1024 * 1024;
pub const OUTBOUND_TRANSFER_LOOKUP_MAX_ATTEMPTS: u32 = 1024;
pub const OUTBOUND_TRANSFER_COMMENT_MAX_BYTES: usize = 960;
pub const OUTBOUND_TRANSFER_SERVER_ID_MAX_BYTES: usize = 1024;
pub const OUTBOUND_TRANSFER_DOMAIN_MAX_BYTES: usize = 1024;
pub const OUTBOUND_TRANSFER_SERVER_COMMENT_MAX_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutboundTransferHandoff {
    Preparation,
    Possible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutboundTransferTerminal {
    None,
    Confirmed,
    Replaced,
    SequenceNumberConsumed,
    Expired,
    Superseded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct OutboundTransferProjection {
    pub id: String,
    pub counterparty: String,
    pub counterparty_name: String,
    pub comment: String,
    pub collectible: String,
    pub counterparty_actor_id: Option<ActorId>,
    pub amount_nano: i64,
    pub fee_nano: Option<i64>,
    pub date_ms: Option<i64>,
    pub peer_transfer: bool,
    pub failed: bool,
    pub comment_encrypted: bool,
    pub gasless: bool,
    pub counterparty_bounceable: bool,
}

impl Default for OutboundTransferProjection {
    fn default() -> Self {
        Self {
            id: String::new(),
            counterparty: String::new(),
            counterparty_name: String::new(),
            comment: String::new(),
            collectible: String::new(),
            counterparty_actor_id: None,
            amount_nano: 0,
            fee_nano: None,
            date_ms: None,
            peer_transfer: false,
            failed: false,
            comment_encrypted: false,
            gasless: false,
            counterparty_bounceable: false,
        }
    }
}

impl OutboundTransferProjection {
    pub fn validate(&self) -> Result<(), OutboundTransferError> {
        if self.id.trim().is_empty()
            || self.id.as_bytes().len() > OUTBOUND_TRANSFER_SERVER_ID_MAX_BYTES
            || (!self.counterparty.is_empty()
                && (self.counterparty.as_bytes().len() > 128
                    || validate_wallet_address(&self.counterparty).is_err()))
            || self.counterparty_name.as_bytes().len() > OUTBOUND_TRANSFER_DOMAIN_MAX_BYTES
            || self.comment.as_bytes().len() > OUTBOUND_TRANSFER_SERVER_COMMENT_MAX_BYTES
            || (self.comment_encrypted && !self.comment.is_empty())
            || (!self.collectible.is_empty()
                && (self.collectible.as_bytes().len() > 128
                    || validate_wallet_address(&self.collectible).is_err()))
            || self.counterparty_actor_id.as_ref().is_some_and(|actor| !actor.is_valid())
            || (self.counterparty_actor_id.is_some()
                && !self.peer_transfer
                && self.collectible.is_empty())
            || self.amount_nano < 0
            || self.date_ms.is_some_and(|value| value <= 0)
        {
            return Err(OutboundTransferError::InvalidProjection);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboundTransferRecord {
    pub record_id: String,
    pub network: u8,
    pub address: String,
    pub public_key: Vec<u8>,
    pub operation_id: String,
    pub destination: String,
    pub comment: String,
    pub collectible: Option<String>,
    #[serde(default)]
    pub recipient_actor_id: Option<ActorId>,
    #[serde(default)]
    pub served: Option<OutboundTransferProjection>,
    pub amount_nano: i64,
    pub posted_at_ms: i64,
    pub handoff: OutboundTransferHandoff,
    pub terminal: OutboundTransferTerminal,
    pub message_token: Option<Vec<u8>>,
    pub confirmed_hash: Option<Vec<u8>>,
    pub lookup_attempts: u32,
    pub lookup_stopped: bool,
    pub paired: bool,
    pub bounce: bool,
}

impl OutboundTransferRecord {
    pub fn validate(&self) -> Result<(), OutboundTransferError> {
        if self.record_id.trim().is_empty()
            || self.record_id.len() > 256
            || self.operation_id.trim().is_empty()
            || self.operation_id.len() > 256
            || !matches!(self.network, 1 | 2)
            || self.address.trim().is_empty()
            || self.address.len() > 128
            || self.public_key.len() != 32
            || self.destination.trim().is_empty()
            || self.destination.len() > 128
            || self.comment.as_bytes().len() > OUTBOUND_TRANSFER_COMMENT_MAX_BYTES
            || self.collectible.as_deref().is_some_and(|value| value.len() > 128)
            || self.recipient_actor_id.as_ref().is_some_and(|actor| !actor.is_valid())
            || self.served.as_ref().is_some_and(|projection| projection.validate().is_err())
            || self.amount_nano <= 0
            || self.posted_at_ms <= 0
            || self.lookup_attempts > OUTBOUND_TRANSFER_LOOKUP_MAX_ATTEMPTS
            || self
                .message_token
                .as_ref()
                .is_some_and(|token| token.is_empty() || token.len() > OUTBOUND_TRANSFER_TOKEN_MAX_BYTES)
            || self
                .confirmed_hash
                .as_ref()
                .is_some_and(|hash| hash.len() != 32)
            || (self.confirmed_hash.is_some()
                && self.terminal != OutboundTransferTerminal::Confirmed)
            || (self.lookup_attempts > 0 && self.message_token.is_none())
        {
            return Err(OutboundTransferError::InvalidRecord);
        }
        if self.handoff == OutboundTransferHandoff::Preparation
            && (self.message_token.is_some()
                || self.terminal != OutboundTransferTerminal::None
                || self.served.is_some()
                || self.lookup_attempts != 0
                || self.lookup_stopped)
        {
            return Err(OutboundTransferError::InvalidRecord);
        }
        Ok(())
    }

    fn identity_matches(&self, other: &Self) -> bool {
        self.network == other.network
            && self.address == other.address
            && self.public_key == other.public_key
            && self.operation_id == other.operation_id
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct OutboundTransferJournal {
    pub records: Vec<OutboundTransferRecord>,
}

impl OutboundTransferJournal {
    pub fn prepare(
        &mut self,
        record: OutboundTransferRecord,
    ) -> Result<(), OutboundTransferError> {
        record.validate()?;
        if record.handoff != OutboundTransferHandoff::Preparation
            || record.terminal != OutboundTransferTerminal::None
        {
            return Err(OutboundTransferError::InvalidTransition);
        }
        if self.records.len() >= OUTBOUND_TRANSFER_MAX_RECORDS {
            return Err(OutboundTransferError::CapacityExceeded);
        }
        if self.records.iter().any(|item| {
            item.record_id == record.record_id || item.identity_matches(&record)
        }) {
            return Err(OutboundTransferError::DuplicateIdentity);
        }
        let mut candidate = self.records.clone();
        candidate.push(record);
        require_transfer_journal_size(&candidate)?;
        self.records = candidate;
        Ok(())
    }

    /// The durable owner must commit this transition before the network
    /// adapter may submit the already-signed transfer. From this point on a
    /// transport failure is "submission unknown" and recovery/lookup owns the
    /// outcome; a caller must never blindly resubmit it.
    pub fn mark_handoff_possible(
        &mut self,
        record_id: &str,
        message_token: Vec<u8>,
    ) -> Result<(), OutboundTransferError> {
        if message_token.is_empty() || message_token.len() > OUTBOUND_TRANSFER_TOKEN_MAX_BYTES {
            return Err(OutboundTransferError::InvalidToken);
        }
        let index = self.index(record_id)?;
        let mut candidate = self.records.clone();
        let record = &mut candidate[index];
        if record.handoff != OutboundTransferHandoff::Preparation
            || record.terminal != OutboundTransferTerminal::None
        {
            return Err(OutboundTransferError::InvalidTransition);
        }
        record.handoff = OutboundTransferHandoff::Possible;
        record.message_token = Some(message_token);
        record.validate()?;
        require_transfer_journal_size(&candidate)?;
        self.records = candidate;
        Ok(())
    }

    pub fn record_served_projection(
        &mut self,
        record_id: &str,
        projection: OutboundTransferProjection,
    ) -> Result<(), OutboundTransferError> {
        projection.validate()?;
        let index = self.index(record_id)?;
        let mut candidate = self.records.clone();
        let record = &mut candidate[index];
        if record.handoff != OutboundTransferHandoff::Possible
            || record.terminal != OutboundTransferTerminal::None
        {
            return Err(OutboundTransferError::InvalidTransition);
        }
        record.served = Some(projection);
        record.validate()?;
        require_transfer_journal_size(&candidate)?;
        self.records = candidate;
        Ok(())
    }

    pub fn note_lookup_attempt(&mut self, record_id: &str) -> Result<u32, OutboundTransferError> {
        let index = self.index(record_id)?;
        let record = &mut self.records[index];
        if record.handoff != OutboundTransferHandoff::Possible
            || record.terminal != OutboundTransferTerminal::None
            || record.message_token.is_none()
            || record.lookup_stopped
        {
            return Err(OutboundTransferError::InvalidTransition);
        }
        if record.lookup_attempts >= OUTBOUND_TRANSFER_LOOKUP_MAX_ATTEMPTS {
            record.lookup_stopped = true;
            return Err(OutboundTransferError::LookupLimitReached);
        }
        record.lookup_attempts += 1;
        Ok(record.lookup_attempts)
    }

    pub fn stop_lookup(&mut self, record_id: &str) -> Result<(), OutboundTransferError> {
        let index = self.index(record_id)?;
        let record = &mut self.records[index];
        if record.handoff != OutboundTransferHandoff::Possible
            || record.terminal != OutboundTransferTerminal::None
        {
            return Err(OutboundTransferError::InvalidTransition);
        }
        record.lookup_stopped = true;
        Ok(())
    }

    pub fn settle(
        &mut self,
        record_id: &str,
        terminal: OutboundTransferTerminal,
        confirmed_hash: Option<Vec<u8>>,
    ) -> Result<(), OutboundTransferError> {
        if terminal == OutboundTransferTerminal::None {
            return Err(OutboundTransferError::InvalidTransition);
        }
        if terminal == OutboundTransferTerminal::Confirmed {
            if confirmed_hash.as_ref().is_none_or(|hash| hash.len() != 32) {
                return Err(OutboundTransferError::InvalidConfirmation);
            }
        } else if confirmed_hash.is_some() {
            return Err(OutboundTransferError::InvalidConfirmation);
        }
        let index = self.index(record_id)?;
        let record = &mut self.records[index];
        if record.handoff != OutboundTransferHandoff::Possible
            || record.terminal != OutboundTransferTerminal::None
        {
            return Err(OutboundTransferError::InvalidTransition);
        }
        record.terminal = terminal;
        record.confirmed_hash = confirmed_hash;
        record.lookup_stopped = true;
        record.validate()
    }

    pub fn submission_unknown(&self) -> Vec<&OutboundTransferRecord> {
        self.records
            .iter()
            .filter(|record| {
                record.handoff == OutboundTransferHandoff::Possible
                    && record.terminal == OutboundTransferTerminal::None
            })
            .collect()
    }

    fn index(&self, record_id: &str) -> Result<usize, OutboundTransferError> {
        self.records
            .iter()
            .position(|record| record.record_id == record_id)
            .ok_or(OutboundTransferError::RecordNotFound)
    }
}

fn require_transfer_journal_size(
    records: &[OutboundTransferRecord],
) -> Result<(), OutboundTransferError> {
    if records.len() > OUTBOUND_TRANSFER_MAX_RECORDS {
        return Err(OutboundTransferError::CapacityExceeded);
    }
    let bytes = serde_json::to_vec(records)
        .map_err(|_| OutboundTransferError::InvalidRecord)?;
    if bytes.len() > OUTBOUND_TRANSFER_MAX_BYTES {
        return Err(OutboundTransferError::CapacityExceeded);
    }
    Ok(())
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OutboundTransferError {
    #[error("outbound transfer journal record is invalid")]
    InvalidRecord,
    #[error("outbound transfer served projection is invalid")]
    InvalidProjection,
    #[error("outbound transfer journal identity already exists")]
    DuplicateIdentity,
    #[error("outbound transfer journal capacity was exceeded")]
    CapacityExceeded,
    #[error("outbound transfer journal record was not found")]
    RecordNotFound,
    #[error("outbound transfer journal transition is invalid")]
    InvalidTransition,
    #[error("outbound transfer journal token is invalid")]
    InvalidToken,
    #[error("outbound transfer lookup limit was reached")]
    LookupLimitReached,
    #[error("outbound transfer confirmation is invalid")]
    InvalidConfirmation,
}


pub const WALLET_ADDRESS_RESOLVE_BATCH_MAX: usize = 100;
pub const WALLET_ADDRESS_MAX_BYTES: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WalletAddressKnowledge {
    Unknown,
    Absent,
    Known,
}

impl Default for WalletAddressKnowledge {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletUserAddress {
    pub knowledge: WalletAddressKnowledge,
    pub address: String,
    pub public_key: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletAddressOwner {
    pub actor_id: Option<ActorId>,
    pub address: String,
    pub public_key: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WalletAddressDirectory {
    pub users: BTreeMap<ActorId, WalletUserAddress>,
    pub owners: BTreeMap<String, WalletAddressOwner>,
    pub newest_answer_serial: BTreeMap<ActorId, u64>,
    pub service_unavailable: bool,
}

impl WalletAddressDirectory {
    pub fn apply_user_answer(
        &mut self,
        actor_id: ActorId,
        serial: u64,
        address: Option<String>,
        public_key: Vec<u8>,
    ) -> Result<bool, WalletAddressDirectoryError> {
        if serial == 0 {
            return Err(WalletAddressDirectoryError::InvalidSerial);
        }
        if self
            .newest_answer_serial
            .get(&actor_id)
            .is_some_and(|known| *known > serial)
        {
            return Ok(false);
        }

        let value = match address {
            Some(address) => {
                validate_wallet_address(&address)?;
                if !public_key.is_empty() && public_key.len() != 32 {
                    return Err(WalletAddressDirectoryError::InvalidPublicKey);
                }
                WalletUserAddress {
                    knowledge: WalletAddressKnowledge::Known,
                    address,
                    public_key,
                }
            }
            None => {
                if !public_key.is_empty() {
                    return Err(WalletAddressDirectoryError::InvalidPublicKey);
                }
                WalletUserAddress {
                    knowledge: WalletAddressKnowledge::Absent,
                    address: String::new(),
                    public_key: Vec::new(),
                }
            }
        };
        self.newest_answer_serial.insert(actor_id.clone(), serial);
        if value.knowledge == WalletAddressKnowledge::Known {
            self.owners
                .entry(value.address.clone())
                .and_modify(|owner| {
                    owner.actor_id = Some(actor_id.clone());
                    if !value.public_key.is_empty() {
                        owner.public_key = value.public_key.clone();
                    }
                })
                .or_insert_with(|| WalletAddressOwner {
                    actor_id: Some(actor_id.clone()),
                    address: value.address.clone(),
                    public_key: value.public_key.clone(),
                });
        }
        self.users.insert(actor_id, value);
        Ok(true)
    }

    pub fn apply_owner_answer(
        &mut self,
        address: String,
        actor_id: Option<ActorId>,
        public_key: Vec<u8>,
    ) -> Result<(), WalletAddressDirectoryError> {
        validate_wallet_address(&address)?;
        if !public_key.is_empty() && public_key.len() != 32 {
            return Err(WalletAddressDirectoryError::InvalidPublicKey);
        }
        let owner = WalletAddressOwner {
            actor_id: actor_id.clone(),
            address: address.clone(),
            public_key: public_key.clone(),
        };
        self.owners.insert(address.clone(), owner);
        if let Some(actor_id) = actor_id {
            if let Some(user) = self.users.get_mut(&actor_id) {
                if user.knowledge == WalletAddressKnowledge::Known && user.address == address {
                    if !public_key.is_empty() {
                        user.public_key = public_key;
                    }
                }
            }
        }
        Ok(())
    }

    /// Unavailable is an account/service fact. It deliberately does not turn
    /// any unresolved actor into Absent, because the upstream contract keeps
    /// those two facts separate.
    pub fn set_service_unavailable(&mut self, unavailable: bool) {
        self.service_unavailable = unavailable;
    }

    pub fn known(&self, actor_id: &ActorId) -> WalletUserAddress {
        self.users.get(actor_id).cloned().unwrap_or_default()
    }

    pub fn owner(&self, address: &str) -> Option<&WalletAddressOwner> {
        self.owners.get(address)
    }
}

fn validate_wallet_address(address: &str) -> Result<(), WalletAddressDirectoryError> {
    if address.is_empty()
        || address.trim() != address
        || address.as_bytes().len() > WALLET_ADDRESS_MAX_BYTES
        || address.chars().any(char::is_control)
    {
        return Err(WalletAddressDirectoryError::InvalidAddress);
    }
    Ok(())
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WalletAddressDirectoryError {
    #[error("wallet address directory serial is invalid")]
    InvalidSerial,
    #[error("wallet address is invalid")]
    InvalidAddress,
    #[error("wallet address public key is invalid")]
    InvalidPublicKey,
}

pub const WALLET_SPONSORED_FEE_MIN_NANO_DEFAULT: i64 = 100_000_000;
pub const WALLET_SPONSORED_FEE_REFRESH_MS: i64 = 60 * 1_000;
pub const WALLET_SPONSORED_FEE_REFRESH_AHEAD_MS: i64 = 10 * 1_000;
pub const WALLET_SPONSORED_FEE_REQUEST_TIMEOUT_MS: i64 = 15 * 1_000;
pub const WALLET_SPONSORED_FEE_RETRY_MS: i64 = 15 * 1_000;
const WALLET_SPONSORED_FEE_MIN_NANO_MAX: i64 = 1_i64 << 53;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletTransferIdentity {
    pub network: u8,
    pub address: String,
    pub public_key: Vec<u8>,
    pub revision: u64,
}

impl WalletTransferIdentity {
    pub(crate) fn validate(&self) -> Result<(), WalletSponsoredFeeError> {
        if !matches!(self.network, 1 | 2)
            || self.public_key.len() != 32
            || validate_wallet_address(&self.address).is_err()
        {
            return Err(WalletSponsoredFeeError::InvalidIdentity);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletSponsoredFeeInfo {
    pub relayer_address: String,
    pub min_amount_nano: i64,
    pub reset_at_ms: i64,
    pub left: i32,
    pub available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletSponsoredFeeTerms {
    pub identity: Option<WalletTransferIdentity>,
    pub info: Option<WalletSponsoredFeeInfo>,
    pub transfer_min_nano: i64,
    pub configured_min_nano: i64,
    pub effective_min_nano: i64,
    pub revision: u64,
    pub fresh: bool,
    pub usable: bool,
}

impl Default for WalletSponsoredFeeTerms {
    fn default() -> Self {
        Self {
            identity: None,
            info: None,
            transfer_min_nano: WALLET_SPONSORED_FEE_MIN_NANO_DEFAULT,
            configured_min_nano: WALLET_SPONSORED_FEE_MIN_NANO_DEFAULT,
            effective_min_nano: WALLET_SPONSORED_FEE_MIN_NANO_DEFAULT,
            revision: 0,
            fresh: false,
            usable: false,
        }
    }
}

impl WalletSponsoredFeeTerms {
    pub fn eligible(&self, amount_nano: i64, destination: &str) -> bool {
        self.usable
            && amount_nano > 0
            && amount_nano >= self.effective_min_nano
            && validate_wallet_address(destination).is_ok()
            && self
                .identity
                .as_ref()
                .is_some_and(|identity| destination != identity.address)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletSponsoredFeeRequest {
    pub serial: u64,
    pub network_generation: u64,
    pub identity: WalletTransferIdentity,
    pub requested_at_ms: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WalletSponsoredFeeState {
    pub network_generation: u64,
    pub terms: WalletSponsoredFeeTerms,
    pub active_request: Option<WalletSponsoredFeeRequest>,
    pub last_requested_at_ms: Option<i64>,
    pub expires_at_ms: Option<i64>,
    pub last_failure_at_ms: Option<i64>,
    pub refresh_wanted: bool,
    next_serial: u64,
}

impl WalletSponsoredFeeState {
    pub fn reset_for_network_generation(&mut self, network_generation: u64) {
        *self = Self {
            network_generation,
            ..Self::default()
        };
    }

    pub fn sync_context(
        &mut self,
        network_generation: u64,
        identity: WalletTransferIdentity,
        transfer_min_nano: i64,
        configured_min_nano: i64,
        now_ms: i64,
    ) -> Result<(), WalletSponsoredFeeError> {
        require_sponsored_timestamp(now_ms)?;
        if network_generation == 0 {
            return Err(WalletSponsoredFeeError::StaleNetworkGeneration {
                current: self.network_generation,
                received: network_generation,
            });
        }
        identity.validate()?;
        if self.network_generation != 0 && self.network_generation != network_generation {
            return Err(WalletSponsoredFeeError::StaleNetworkGeneration {
                current: self.network_generation,
                received: network_generation,
            });
        }
        if self.network_generation == 0 {
            self.network_generation = network_generation;
        }

        if self.terms.identity.as_ref() != Some(&identity) {
            let previous_revision = self.terms.revision;
            self.active_request = None;
            self.last_requested_at_ms = None;
            self.expires_at_ms = None;
            self.last_failure_at_ms = None;
            self.refresh_wanted = false;
            self.next_serial = 0;
            self.terms = WalletSponsoredFeeTerms {
                identity: Some(identity),
                revision: previous_revision.saturating_add(1),
                ..WalletSponsoredFeeTerms::default()
            };
        }

        let transfer_min_nano = canonical_sponsored_minimum(transfer_min_nano);
        let configured_min_nano = canonical_sponsored_minimum(configured_min_nano);
        if self.terms.transfer_min_nano != transfer_min_nano
            || self.terms.configured_min_nano != configured_min_nano
        {
            self.terms.transfer_min_nano = transfer_min_nano;
            self.terms.configured_min_nano = configured_min_nano;
            self.terms.revision = self.terms.revision.saturating_add(1);
        }
        self.refresh_liveness(now_ms)
    }

    pub fn request_due(&self, now_ms: i64) -> bool {
        if now_ms < 0 || self.terms.identity.is_none() {
            return false;
        }
        if let Some(request) = &self.active_request {
            if now_ms.saturating_sub(request.requested_at_ms)
                < WALLET_SPONSORED_FEE_REQUEST_TIMEOUT_MS
            {
                return false;
            }
        }
        let fresh_now = self.fresh_at(now_ms);
        let expiring = fresh_now
            && self
                .expires_at_ms
                .is_some_and(|expires| now_ms >= expires.saturating_sub(WALLET_SPONSORED_FEE_REFRESH_AHEAD_MS));
        let retry_allowed = self
            .last_requested_at_ms
            .is_none_or(|last| now_ms.saturating_sub(last) >= WALLET_SPONSORED_FEE_RETRY_MS);
        retry_allowed && (!fresh_now || expiring || self.refresh_wanted)
    }

    pub fn begin_request(
        &mut self,
        network_generation: u64,
        identity: WalletTransferIdentity,
        transfer_min_nano: i64,
        configured_min_nano: i64,
        now_ms: i64,
        force: bool,
    ) -> Result<Option<u64>, WalletSponsoredFeeError> {
        self.sync_context(
            network_generation,
            identity.clone(),
            transfer_min_nano,
            configured_min_nano,
            now_ms,
        )?;
        self.refresh_wanted |= force;
        if !self.request_due(now_ms) {
            return Ok(None);
        }
        self.next_serial = self
            .next_serial
            .checked_add(1)
            .ok_or(WalletSponsoredFeeError::SerialOverflow)?;
        let serial = self.next_serial;
        self.active_request = Some(WalletSponsoredFeeRequest {
            serial,
            network_generation,
            identity,
            requested_at_ms: now_ms,
        });
        self.last_requested_at_ms = Some(now_ms);
        self.refresh_wanted = false;
        Ok(Some(serial))
    }

    pub fn apply_info(
        &mut self,
        serial: u64,
        network_generation: u64,
        identity: &WalletTransferIdentity,
        info: WalletSponsoredFeeInfo,
        now_ms: i64,
    ) -> Result<(), WalletSponsoredFeeError> {
        require_sponsored_timestamp(now_ms)?;
        self.require_request(serial, network_generation, identity)?;
        self.active_request = None;
        self.last_failure_at_ms = None;
        self.expires_at_ms = Some(
            now_ms
                .checked_add(WALLET_SPONSORED_FEE_REFRESH_MS)
                .ok_or(WalletSponsoredFeeError::InvalidTimestamp)?,
        );
        self.terms.identity = Some(identity.clone());
        self.terms.info = Some(info);
        self.terms.revision = self.terms.revision.saturating_add(1);
        self.refresh_liveness(now_ms)
    }

    pub fn fail_request(
        &mut self,
        serial: u64,
        network_generation: u64,
        identity: &WalletTransferIdentity,
        now_ms: i64,
    ) -> Result<(), WalletSponsoredFeeError> {
        require_sponsored_timestamp(now_ms)?;
        self.require_request(serial, network_generation, identity)?;
        self.active_request = None;
        self.expires_at_ms = None;
        self.last_failure_at_ms = Some(now_ms);
        self.refresh_wanted = true;
        self.terms.revision = self.terms.revision.saturating_add(1);
        self.recompute_terms(now_ms);
        Ok(())
    }

    pub fn refresh_liveness(&mut self, now_ms: i64) -> Result<(), WalletSponsoredFeeError> {
        require_sponsored_timestamp(now_ms)?;
        if self.active_request.as_ref().is_some_and(|request| {
            now_ms.saturating_sub(request.requested_at_ms)
                >= WALLET_SPONSORED_FEE_REQUEST_TIMEOUT_MS
        }) {
            self.active_request = None;
            self.expires_at_ms = None;
            self.last_failure_at_ms = Some(now_ms);
            self.refresh_wanted = true;
            self.terms.revision = self.terms.revision.saturating_add(1);
        }
        if !self.fresh_at(now_ms) {
            self.expires_at_ms = None;
        }
        self.recompute_terms(now_ms);
        Ok(())
    }

    fn fresh_at(&self, now_ms: i64) -> bool {
        self.terms.identity.is_some()
            && self.terms.info.is_some()
            && self.expires_at_ms.is_some_and(|expires| expires > now_ms)
            && self
                .terms
                .info
                .as_ref()
                .is_none_or(|info| info.reset_at_ms <= 0 || info.reset_at_ms > now_ms)
    }

    fn recompute_terms(&mut self, now_ms: i64) {
        let before = self.terms.clone();
        let mut effective = self
            .terms
            .transfer_min_nano
            .max(self.terms.configured_min_nano);
        if let Some(info) = &self.terms.info {
            if info.min_amount_nano > 0 {
                effective = effective.max(info.min_amount_nano);
            }
        }
        self.terms.effective_min_nano = effective;
        self.terms.fresh = self.fresh_at(now_ms);
        self.terms.usable = self.terms.fresh
            && self.terms.info.as_ref().is_some_and(|info| {
                info.available
                    && info.left > 0
                    && info.reset_at_ms >= 0
                    && info.min_amount_nano > 0
                    && self.terms.identity.as_ref().is_some_and(|identity| {
                        identity.network == 1
                            && validate_wallet_address(&info.relayer_address).is_ok()
                    })
            });
        if self.terms != before {
            self.terms.revision = before.revision.saturating_add(1);
        }
    }

    fn require_request(
        &self,
        serial: u64,
        network_generation: u64,
        identity: &WalletTransferIdentity,
    ) -> Result<(), WalletSponsoredFeeError> {
        if self.network_generation != network_generation {
            return Err(WalletSponsoredFeeError::StaleNetworkGeneration {
                current: self.network_generation,
                received: network_generation,
            });
        }
        match &self.active_request {
            Some(request)
                if request.serial == serial
                    && request.network_generation == network_generation
                    && &request.identity == identity
                    && self.terms.identity.as_ref() == Some(identity) =>
            {
                Ok(())
            }
            _ => Err(WalletSponsoredFeeError::StaleRequest),
        }
    }
}

fn canonical_sponsored_minimum(value: i64) -> i64 {
    if (1..=WALLET_SPONSORED_FEE_MIN_NANO_MAX).contains(&value) {
        value
    } else {
        WALLET_SPONSORED_FEE_MIN_NANO_DEFAULT
    }
}

fn require_sponsored_timestamp(value: i64) -> Result<(), WalletSponsoredFeeError> {
    if value >= 0 {
        Ok(())
    } else {
        Err(WalletSponsoredFeeError::InvalidTimestamp)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WalletSponsoredFeeError {
    #[error("wallet sponsored-fee identity is invalid")]
    InvalidIdentity,
    #[error("wallet sponsored-fee timestamp is invalid")]
    InvalidTimestamp,
    #[error("wallet sponsored-fee request serial overflowed")]
    SerialOverflow,
    #[error("wallet sponsored-fee network generation is stale: current {current}, received {received}")]
    StaleNetworkGeneration { current: u64, received: u64 },
    #[error("wallet sponsored-fee response does not own the active request")]
    StaleRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletTransferQuote {
    pub network_generation: u64,
    pub identity: WalletTransferIdentity,
    pub sponsored_terms_revision: u64,
    pub amount_nano: i64,
    pub destination: String,
    pub fee_nano: i64,
    pub paired: bool,
}

impl WalletSponsoredFeeState {
    pub fn quote_transfer(
        &mut self,
        network_generation: u64,
        identity: WalletTransferIdentity,
        transfer_min_nano: i64,
        configured_min_nano: i64,
        amount_nano: i64,
        destination: String,
        fee_nano: i64,
        now_ms: i64,
    ) -> Result<WalletTransferQuote, WalletTransferQuoteError> {
        self.sync_context(
            network_generation,
            identity.clone(),
            transfer_min_nano,
            configured_min_nano,
            now_ms,
        )?;
        self.refresh_liveness(now_ms)?;
        if amount_nano <= 0 || fee_nano < 0 || validate_wallet_address(&destination).is_err() {
            return Err(WalletTransferQuoteError::InvalidTransfer);
        }
        if amount_nano < transfer_min_nano {
            return Err(WalletTransferQuoteError::AmountBelowMinimum);
        }
        let paired = self.terms.eligible(amount_nano, &destination);
        Ok(WalletTransferQuote {
            network_generation,
            identity,
            sponsored_terms_revision: self.terms.revision,
            amount_nano,
            destination,
            fee_nano,
            paired,
        })
    }

    pub fn authorize_quote(
        &mut self,
        quote: &WalletTransferQuote,
        balance_nano: i64,
        now_ms: i64,
    ) -> Result<(), WalletTransferQuoteError> {
        self.refresh_liveness(now_ms)?;
        if balance_nano < 0 || quote.amount_nano <= 0 || quote.fee_nano < 0 {
            return Err(WalletTransferQuoteError::InvalidTransfer);
        }
        if quote.network_generation != self.network_generation
            || self.terms.identity.as_ref() != Some(&quote.identity)
        {
            return Err(WalletTransferQuoteError::StaleIdentity);
        }
        if quote.sponsored_terms_revision != self.terms.revision {
            return Err(WalletTransferQuoteError::QuoteExpired);
        }
        let paired_now = self
            .terms
            .eligible(quote.amount_nano, &quote.destination);
        if paired_now != quote.paired {
            return Err(WalletTransferQuoteError::QuoteExpired);
        }
        if quote.amount_nano > balance_nano {
            return Err(WalletTransferQuoteError::InsufficientBalance);
        }
        if !quote.paired {
            let required = quote
                .amount_nano
                .checked_add(quote.fee_nano)
                .ok_or(WalletTransferQuoteError::InsufficientFees)?;
            if required > balance_nano {
                return Err(WalletTransferQuoteError::InsufficientFees);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WalletTransferQuoteError {
    #[error("wallet transfer quote is invalid")]
    InvalidTransfer,
    #[error("wallet transfer amount is below the configured minimum")]
    AmountBelowMinimum,
    #[error("wallet transfer quote identity or network is stale")]
    StaleIdentity,
    #[error("wallet transfer quote has expired")]
    QuoteExpired,
    #[error("wallet transfer balance is insufficient")]
    InsufficientBalance,
    #[error("wallet transfer balance cannot reserve the normal fee")]
    InsufficientFees,
    #[error(transparent)]
    SponsoredFee(#[from] WalletSponsoredFeeError),
    #[error(transparent)]
    OutboundTransfer(#[from] OutboundTransferError),
}

pub const WALLET_LIVE_STATE_REFRESH_MS: i64 = 60 * 1_000;
pub const WALLET_LIVE_STREAM_RESYNC_MS: i64 = 30 * 1_000;
pub const WALLET_LIVE_FAILURES_BEFORE_UNREACHABLE: u8 = 2;
pub const WALLET_LIVE_MAX_HIDDEN_PAGES: u8 = 20;
pub const WALLET_STREAM_COALESCE_MS: i64 = 250;
pub const WALLET_STREAM_HISTORY_RECHECK_MS: i64 = 3_000;
pub const WALLET_STREAM_HISTORY_FINAL_RECHECK_MS: i64 = 10_000;
pub const WALLET_STREAM_KEEPALIVE_MS: i64 = 10_000;
pub const WALLET_STREAM_STALL_MS: i64 = 30_000;
pub const WALLET_STREAM_STABLE_MS: i64 = 60_000;
pub const WALLET_STREAM_RENEW_MARGIN_MS: i64 = 60_000;
pub const WALLET_STREAM_MAX_RENEW_MS: i64 = 30 * 60 * 1_000;
pub const WALLET_STREAM_RETRY_MS: [i64; 7] = [500, 1_000, 2_000, 4_000, 8_000, 15_000, 30_000];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WalletStreamPhase {
    #[default]
    Idle,
    Acquiring,
    Connecting,
    Live,
    Backoff,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletStreamRefresh {
    pub state: bool,
    pub history: bool,
    pub collectibles: bool,
}

impl WalletStreamRefresh {
    pub fn merge(&mut self, other: Self) {
        self.state |= other.state;
        self.history |= other.history;
        self.collectibles |= other.collectibles;
    }

    pub fn any(self) -> bool {
        self.state || self.history || self.collectibles
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletStreamState {
    pub generation: u64,
    pub address: String,
    pub phase: WalletStreamPhase,
    pub retry_attempt: u8,
    pub live_since_ms: Option<i64>,
    pub last_activity_ms: Option<i64>,
    pub retry_at_ms: Option<i64>,
    pub renew_at_ms: Option<i64>,
    pub coalesce_due_ms: Option<i64>,
    pub history_recheck_due_ms: Option<i64>,
    pub history_rechecked_once: bool,
    pub wanted: WalletStreamRefresh,
}

impl WalletStreamState {
    pub fn start(&mut self, address: &str, now_ms: i64) -> Result<u64, WalletLiveError> {
        require_live_timestamp(now_ms)?;
        validate_wallet_address(address).map_err(|_| WalletLiveError::InvalidStreamAddress)?;
        if self.phase != WalletStreamPhase::Idle && self.address == address {
            return Ok(self.generation);
        }
        self.bump_generation()?;
        self.address = address.to_string();
        self.phase = WalletStreamPhase::Acquiring;
        self.retry_attempt = 0;
        self.live_since_ms = None;
        self.last_activity_ms = None;
        self.retry_at_ms = None;
        self.renew_at_ms = None;
        self.coalesce_due_ms = None;
        self.history_recheck_due_ms = None;
        self.history_rechecked_once = false;
        self.wanted = WalletStreamRefresh::default();
        Ok(self.generation)
    }

    pub fn stop(&mut self) -> Result<u64, WalletLiveError> {
        self.bump_generation()?;
        self.address.clear();
        self.phase = WalletStreamPhase::Idle;
        self.retry_attempt = 0;
        self.live_since_ms = None;
        self.last_activity_ms = None;
        self.retry_at_ms = None;
        self.renew_at_ms = None;
        self.coalesce_due_ms = None;
        self.history_recheck_due_ms = None;
        self.history_rechecked_once = false;
        self.wanted = WalletStreamRefresh::default();
        Ok(self.generation)
    }

    pub fn apply_url(
        &mut self,
        generation: u64,
        now_ms: i64,
        expires_in_ms: i64,
    ) -> Result<u64, WalletLiveError> {
        self.require_generation(generation)?;
        require_live_timestamp(now_ms)?;
        if self.phase != WalletStreamPhase::Acquiring || expires_in_ms <= 0 {
            return Err(WalletLiveError::InvalidStreamTransition);
        }
        let minimal = (expires_in_ms / 2).min(WALLET_STREAM_MAX_RENEW_MS);
        let renew_delay = (expires_in_ms - WALLET_STREAM_RENEW_MARGIN_MS)
            .clamp(minimal, WALLET_STREAM_MAX_RENEW_MS);
        self.renew_at_ms = Some(now_ms.saturating_add(renew_delay));
        self.bump_generation()?;
        self.phase = WalletStreamPhase::Connecting;
        self.last_activity_ms = Some(now_ms);
        self.live_since_ms = None;
        self.retry_at_ms = None;
        Ok(self.generation)
    }

    pub fn connected(&mut self, generation: u64, now_ms: i64) -> Result<(), WalletLiveError> {
        self.require_generation(generation)?;
        require_live_timestamp(now_ms)?;
        if self.phase != WalletStreamPhase::Connecting {
            return Err(WalletLiveError::InvalidStreamTransition);
        }
        self.phase = WalletStreamPhase::Live;
        self.live_since_ms = Some(now_ms);
        self.last_activity_ms = Some(now_ms);
        Ok(())
    }

    pub fn note_activity(&mut self, generation: u64, now_ms: i64) -> Result<(), WalletLiveError> {
        self.require_generation(generation)?;
        require_live_timestamp(now_ms)?;
        if !matches!(self.phase, WalletStreamPhase::Connecting | WalletStreamPhase::Live) {
            return Err(WalletLiveError::InvalidStreamTransition);
        }
        self.last_activity_ms = Some(now_ms);
        Ok(())
    }

    pub fn keepalive_action(
        &self,
        generation: u64,
        now_ms: i64,
    ) -> Result<WalletStreamKeepalive, WalletLiveError> {
        self.require_generation(generation)?;
        require_live_timestamp(now_ms)?;
        if self.phase != WalletStreamPhase::Live {
            return Ok(WalletStreamKeepalive::None);
        }
        let stalled = self
            .last_activity_ms
            .is_some_and(|last| now_ms.saturating_sub(last) > WALLET_STREAM_STALL_MS);
        Ok(if stalled {
            WalletStreamKeepalive::Fail
        } else {
            WalletStreamKeepalive::Ping
        })
    }

    pub fn fail(&mut self, generation: u64, now_ms: i64) -> Result<u64, WalletLiveError> {
        self.require_generation(generation)?;
        require_live_timestamp(now_ms)?;
        if self.phase == WalletStreamPhase::Idle {
            return Err(WalletLiveError::InvalidStreamTransition);
        }
        let stable = self
            .live_since_ms
            .is_some_and(|live| now_ms.saturating_sub(live) > WALLET_STREAM_STABLE_MS);
        if stable {
            self.retry_attempt = 0;
        }
        self.bump_generation()?;
        let index = usize::from(self.retry_attempt)
            .min(WALLET_STREAM_RETRY_MS.len().saturating_sub(1));
        let delay = WALLET_STREAM_RETRY_MS[index];
        self.retry_attempt = self.retry_attempt.saturating_add(1);
        self.phase = WalletStreamPhase::Backoff;
        self.retry_at_ms = Some(now_ms.saturating_add(delay));
        self.renew_at_ms = None;
        self.live_since_ms = None;
        self.last_activity_ms = None;
        self.history_recheck_due_ms = None;
        self.history_rechecked_once = false;
        Ok(self.generation)
    }

    pub fn resume_after_backoff(
        &mut self,
        generation: u64,
        now_ms: i64,
    ) -> Result<bool, WalletLiveError> {
        self.require_generation(generation)?;
        require_live_timestamp(now_ms)?;
        if self.phase != WalletStreamPhase::Backoff {
            return Err(WalletLiveError::InvalidStreamTransition);
        }
        if self.retry_at_ms.is_some_and(|due| now_ms < due) {
            return Ok(false);
        }
        self.phase = WalletStreamPhase::Acquiring;
        self.retry_at_ms = None;
        Ok(true)
    }

    pub fn begin_renew_if_due(
        &mut self,
        generation: u64,
        now_ms: i64,
    ) -> Result<bool, WalletLiveError> {
        self.require_generation(generation)?;
        require_live_timestamp(now_ms)?;
        if !matches!(self.phase, WalletStreamPhase::Live | WalletStreamPhase::Connecting) {
            return Ok(false);
        }
        if self.renew_at_ms.is_some_and(|due| now_ms >= due) {
            self.phase = WalletStreamPhase::Acquiring;
            self.renew_at_ms = None;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn want_refresh(
        &mut self,
        generation: u64,
        now_ms: i64,
        wanted: WalletStreamRefresh,
    ) -> Result<(), WalletLiveError> {
        self.require_generation(generation)?;
        require_live_timestamp(now_ms)?;
        if !wanted.any() {
            return Ok(());
        }
        self.wanted.merge(wanted);
        self.coalesce_due_ms
            .get_or_insert(now_ms.saturating_add(WALLET_STREAM_COALESCE_MS));
        Ok(())
    }

    pub fn note_transaction_event(
        &mut self,
        generation: u64,
        now_ms: i64,
    ) -> Result<(), WalletLiveError> {
        self.want_refresh(
            generation,
            now_ms,
            WalletStreamRefresh {
                state: true,
                history: true,
                collectibles: true,
            },
        )?;
        self.history_rechecked_once = false;
        self.history_recheck_due_ms = Some(now_ms.saturating_add(WALLET_STREAM_HISTORY_RECHECK_MS));
        Ok(())
    }

    pub fn take_refresh_if_due(
        &mut self,
        generation: u64,
        now_ms: i64,
    ) -> Result<Option<WalletStreamRefresh>, WalletLiveError> {
        self.require_generation(generation)?;
        require_live_timestamp(now_ms)?;
        if self.coalesce_due_ms.is_none_or(|due| now_ms < due) {
            return Ok(None);
        }
        self.coalesce_due_ms = None;
        let wanted = std::mem::take(&mut self.wanted);
        Ok(wanted.any().then_some(wanted))
    }

    pub fn take_history_recheck_if_due(
        &mut self,
        generation: u64,
        now_ms: i64,
    ) -> Result<bool, WalletLiveError> {
        self.require_generation(generation)?;
        require_live_timestamp(now_ms)?;
        let Some(due) = self.history_recheck_due_ms else {
            return Ok(false);
        };
        if now_ms < due {
            return Ok(false);
        }
        if self.history_rechecked_once {
            self.history_recheck_due_ms = None;
        } else {
            self.history_rechecked_once = true;
            self.history_recheck_due_ms = Some(
                due.saturating_add(
                    WALLET_STREAM_HISTORY_FINAL_RECHECK_MS - WALLET_STREAM_HISTORY_RECHECK_MS,
                ),
            );
        }
        Ok(true)
    }

    fn bump_generation(&mut self) -> Result<(), WalletLiveError> {
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or(WalletLiveError::StreamGenerationOverflow)?;
        Ok(())
    }

    fn require_generation(&self, generation: u64) -> Result<(), WalletLiveError> {
        if generation != 0 && generation == self.generation {
            Ok(())
        } else {
            Err(WalletLiveError::StaleStreamGeneration {
                current: self.generation,
                received: generation,
            })
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WalletStreamKeepalive {
    #[default]
    None,
    Ping,
    Fail,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WalletLivePresence {
    #[default]
    Unknown,
    Provisioning,
    Missing,
    Unavailable,
    AddressUnreadable,
    Ready,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletLiveState {
    pub generation: u64,
    pub presence: WalletLivePresence,
    pub state_unreachable: bool,
    pub consecutive_state_failures: u8,
    pub last_state_refresh_ms: Option<i64>,
    pub last_stream_resync_ms: Option<i64>,
    pub hidden_pages_without_visible_rows: u8,
    pub stream: WalletStreamState,
}

impl WalletLiveState {
    pub fn begin_generation(&mut self) -> Result<u64, WalletLiveError> {
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or(WalletLiveError::GenerationOverflow)?;
        self.presence = WalletLivePresence::Unknown;
        self.state_unreachable = false;
        self.consecutive_state_failures = 0;
        self.last_state_refresh_ms = None;
        self.last_stream_resync_ms = None;
        self.hidden_pages_without_visible_rows = 0;
        self.stream = WalletStreamState::default();
        Ok(self.generation)
    }

    pub fn apply_presence(
        &mut self,
        generation: u64,
        presence: WalletLivePresence,
        observed_at_ms: i64,
    ) -> Result<(), WalletLiveError> {
        self.require_generation(generation)?;
        require_live_timestamp(observed_at_ms)?;
        self.presence = presence;
        self.state_unreachable = false;
        self.consecutive_state_failures = 0;
        self.last_state_refresh_ms = Some(observed_at_ms);
        Ok(())
    }

    pub fn note_state_failure(
        &mut self,
        generation: u64,
        observed_at_ms: i64,
    ) -> Result<(), WalletLiveError> {
        self.require_generation(generation)?;
        require_live_timestamp(observed_at_ms)?;
        self.consecutive_state_failures = self.consecutive_state_failures.saturating_add(1);
        self.state_unreachable =
            self.consecutive_state_failures >= WALLET_LIVE_FAILURES_BEFORE_UNREACHABLE;
        self.last_state_refresh_ms = Some(observed_at_ms);
        Ok(())
    }

    pub fn state_refresh_due(&self, now_ms: i64) -> bool {
        if now_ms < 0 {
            return false;
        }
        self.last_state_refresh_ms
            .is_none_or(|last| now_ms.saturating_sub(last) >= WALLET_LIVE_STATE_REFRESH_MS)
    }

    pub fn mark_stream_resync(
        &mut self,
        generation: u64,
        observed_at_ms: i64,
    ) -> Result<(), WalletLiveError> {
        self.require_generation(generation)?;
        require_live_timestamp(observed_at_ms)?;
        self.last_stream_resync_ms = Some(observed_at_ms);
        Ok(())
    }

    pub fn stream_resync_due(&self, now_ms: i64) -> bool {
        if now_ms < 0 {
            return false;
        }
        self.last_stream_resync_ms
            .is_none_or(|last| now_ms.saturating_sub(last) >= WALLET_LIVE_STREAM_RESYNC_MS)
    }

    /// Spend the hidden-page budget before dispatching the actual request.
    /// Both viewport-driven load-more and answer-side continuation therefore
    /// consume one shared finite lane and cannot race past the bound.
    pub fn spend_history_page_request(
        &mut self,
        generation: u64,
    ) -> Result<(), WalletLiveError> {
        self.require_generation(generation)?;
        if self.hidden_pages_without_visible_rows >= WALLET_LIVE_MAX_HIDDEN_PAGES {
            return Err(WalletLiveError::HistoryPageBudgetExhausted);
        }
        self.hidden_pages_without_visible_rows =
            self.hidden_pages_without_visible_rows.saturating_add(1);
        Ok(())
    }

    /// Only visible progress refills the automatic walk. Empty answers retain
    /// the budget already spent when their request was issued.
    pub fn note_history_progress(
        &mut self,
        generation: u64,
        visible_rows: usize,
    ) -> Result<(), WalletLiveError> {
        self.require_generation(generation)?;
        if visible_rows > 0 {
            self.hidden_pages_without_visible_rows = 0;
        }
        Ok(())
    }

    pub fn rearm_history_walk(&mut self, generation: u64) -> Result<(), WalletLiveError> {
        self.require_generation(generation)?;
        self.hidden_pages_without_visible_rows = 0;
        Ok(())
    }

    fn require_generation(&self, generation: u64) -> Result<(), WalletLiveError> {
        if generation == self.generation && generation != 0 {
            Ok(())
        } else {
            Err(WalletLiveError::StaleGeneration {
                current: self.generation,
                received: generation,
            })
        }
    }
}

fn require_live_timestamp(value: i64) -> Result<(), WalletLiveError> {
    if value >= 0 {
        Ok(())
    } else {
        Err(WalletLiveError::InvalidTimestamp)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WalletLiveError {
    #[error("wallet live generation overflowed")]
    GenerationOverflow,
    #[error("wallet live generation is stale: current {current}, received {received}")]
    StaleGeneration { current: u64, received: u64 },
    #[error("wallet live timestamp is invalid")]
    InvalidTimestamp,
    #[error("wallet hidden history page budget is exhausted")]
    HistoryPageBudgetExhausted,
    #[error("wallet stream address is invalid")]
    InvalidStreamAddress,
    #[error("wallet stream transition is invalid")]
    InvalidStreamTransition,
    #[error("wallet stream generation overflowed")]
    StreamGenerationOverflow,
    #[error("wallet stream generation is stale: current {current}, received {received}")]
    StaleStreamGeneration { current: u64, received: u64 },
}


#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WalletParkedFunds {
    #[default]
    Checking,
    Empty,
    Funded,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletParkedCheck {
    pub funds: WalletParkedFunds,
    pub balance_nano: i64,
    pub request_serial: Option<u64>,
}

impl Default for WalletParkedCheck {
    fn default() -> Self {
        Self {
            funds: WalletParkedFunds::Checking,
            balance_nano: 0,
            request_serial: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WalletParkedCheckOutcome {
    Unknown,
    Empty,
    Funded { balance_nano: i64 },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletParkedState {
    pub checks: BTreeMap<String, WalletParkedCheck>,
    pub next_serial: u64,
}

impl WalletParkedState {
    pub fn sync_addresses(
        &mut self,
        served_address: &str,
        addresses: &[String],
    ) -> Result<(), WalletParkedError> {
        validate_wallet_address(served_address).map_err(|_| WalletParkedError::InvalidAddress)?;
        let mut wanted = BTreeSet::new();
        for address in addresses {
            validate_wallet_address(address).map_err(|_| WalletParkedError::InvalidAddress)?;
            if address != served_address {
                wanted.insert(address.clone());
            }
        }
        self.checks.retain(|address, _| wanted.contains(address));
        for address in wanted {
            self.checks.entry(address).or_default();
        }
        Ok(())
    }

    pub fn begin_check(
        &mut self,
        address: &str,
        refresh: bool,
        presence: WalletLivePresence,
    ) -> Result<Option<u64>, WalletParkedError> {
        if presence != WalletLivePresence::Ready {
            return Ok(None);
        }
        let check = self
            .checks
            .get(address)
            .ok_or(WalletParkedError::AddressNotTracked)?;
        if check.request_serial.is_some() {
            return Ok(None);
        }
        let eligible = check.funds == WalletParkedFunds::Checking
            || (refresh
                && matches!(
                    check.funds,
                    WalletParkedFunds::Funded | WalletParkedFunds::Unknown
                ));
        if !eligible {
            return Ok(None);
        }
        self.next_serial = self
            .next_serial
            .checked_add(1)
            .ok_or(WalletParkedError::SerialOverflow)?;
        let serial = self.next_serial;
        self.checks
            .get_mut(address)
            .expect("tracked parked wallet disappeared")
            .request_serial = Some(serial);
        Ok(Some(serial))
    }

    pub fn apply_check(
        &mut self,
        address: &str,
        serial: u64,
        outcome: WalletParkedCheckOutcome,
    ) -> Result<(), WalletParkedError> {
        if serial == 0 {
            return Err(WalletParkedError::InvalidSerial);
        }
        if matches!(
            outcome,
            WalletParkedCheckOutcome::Funded { balance_nano } if balance_nano < 0
        ) {
            return Err(WalletParkedError::InvalidBalance);
        }
        let check = self
            .checks
            .get_mut(address)
            .ok_or(WalletParkedError::AddressNotTracked)?;
        if check.request_serial != Some(serial) {
            return Err(WalletParkedError::StaleCheck {
                expected: check.request_serial,
                received: serial,
            });
        }
        check.request_serial = None;
        match outcome {
            WalletParkedCheckOutcome::Unknown => {
                check.funds = WalletParkedFunds::Unknown;
                check.balance_nano = 0;
            }
            WalletParkedCheckOutcome::Empty => {
                check.funds = WalletParkedFunds::Empty;
                check.balance_nano = 0;
            }
            WalletParkedCheckOutcome::Funded { balance_nano } => {
                check.funds = WalletParkedFunds::Funded;
                check.balance_nano = balance_nano;
            }
        }
        Ok(())
    }

    pub fn mark_drop_failed(&mut self, address: &str) -> Result<(), WalletParkedError> {
        let check = self
            .checks
            .get_mut(address)
            .ok_or(WalletParkedError::AddressNotTracked)?;
        if check.funds == WalletParkedFunds::Empty {
            check.funds = WalletParkedFunds::Unknown;
            check.balance_nano = 0;
        }
        Ok(())
    }

    pub fn aggregate_balance_nano(&self) -> Result<Option<i64>, WalletParkedError> {
        let mut total = 0_i64;
        for check in self.checks.values() {
            match check.funds {
                WalletParkedFunds::Unknown => return Ok(None),
                WalletParkedFunds::Funded => {
                    total = total
                        .checked_add(check.balance_nano)
                        .ok_or(WalletParkedError::BalanceOverflow)?;
                }
                WalletParkedFunds::Checking | WalletParkedFunds::Empty => {}
            }
        }
        Ok(Some(total))
    }

    pub fn hidden(&self, address: &str) -> bool {
        self.checks.get(address).is_some_and(|check| {
            matches!(
                check.funds,
                WalletParkedFunds::Checking | WalletParkedFunds::Empty
            )
        })
    }

    pub fn empty_drop_candidates(&self) -> Vec<String> {
        self.checks
            .iter()
            .filter_map(|(address, check)| {
                (check.funds == WalletParkedFunds::Empty).then(|| address.clone())
            })
            .collect()
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WalletParkedError {
    #[error("parked wallet address is invalid")]
    InvalidAddress,
    #[error("parked wallet address is not tracked")]
    AddressNotTracked,
    #[error("parked wallet request serial is invalid")]
    InvalidSerial,
    #[error("parked wallet request serial overflowed")]
    SerialOverflow,
    #[error("parked wallet check is stale: expected {expected:?}, received {received}")]
    StaleCheck {
        expected: Option<u64>,
        received: u64,
    },
    #[error("parked wallet balance is invalid")]
    InvalidBalance,
    #[error("parked wallet balance overflowed")]
    BalanceOverflow,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletRuntimeState {
    pub key_protection: WalletKeyProtectionState,
    pub rates: WalletRateState,
    pub onramp: WalletOnrampState,
    pub panel: WalletPanelState,
    pub outbound_transfers: OutboundTransferJournal,
    pub address_directory: WalletAddressDirectory,
    /// Sponsored-fee service offers, request serials, identity fences,
    /// failures and expiry clocks are live network authority. Persisting
    /// them would replay stale sponsorship after restart.
    #[serde(skip, default)]
    pub sponsored_fees: WalletSponsoredFeeState,
    /// Refresh generations, failure latches and hidden-page pacing are
    /// deliberately runtime-only. Restart must rebuild them from the live
    /// service rather than replaying stale network authority.
    #[serde(skip, default)]
    pub live: WalletLiveState,
    /// Parked-wallet discovery is derived from live custody plus remote balance
    /// checks. Restart must rebuild it rather than trusting stale network facts.
    #[serde(skip, default)]
    pub parked: WalletParkedState,
}

impl WalletRuntimeState {
    pub fn journal_quoted_outbound_transfer(
        &mut self,
        record: OutboundTransferRecord,
        quote: &WalletTransferQuote,
        balance_nano: i64,
        observed_at_ms: i64,
    ) -> Result<(), WalletTransferQuoteError> {
        self.sponsored_fees
            .authorize_quote(quote, balance_nano, observed_at_ms)?;
        if record.network != quote.identity.network
            || record.address != quote.identity.address
            || record.public_key != quote.identity.public_key
            || record.destination != quote.destination
            || record.amount_nano != quote.amount_nano
            || record.paired != quote.paired
        {
            return Err(WalletTransferQuoteError::InvalidTransfer);
        }
        self.outbound_transfers
            .prepare(record)
            .map_err(WalletTransferQuoteError::OutboundTransfer)
    }
}


#[cfg(test)]
mod parked_wallet_tests {
    use super::*;

    #[test]
    fn sync_deduplicates_candidates_and_never_tracks_served_wallet() {
        let mut state = WalletParkedState::default();
        state
            .sync_addresses(
                "EQ-current",
                &[
                    "EQ-current".into(),
                    "EQ-old-a".into(),
                    "EQ-old-a".into(),
                    "EQ-old-b".into(),
                ],
            )
            .unwrap();
        assert_eq!(
            state.checks.keys().cloned().collect::<Vec<_>>(),
            vec!["EQ-old-a".to_string(), "EQ-old-b".to_string()]
        );
        assert!(state.hidden("EQ-old-a"));
        assert_eq!(state.aggregate_balance_nano().unwrap(), Some(0));
    }

    #[test]
    fn checks_are_ready_gated_stale_safe_and_refresh_funded_without_hiding_it() {
        let mut state = WalletParkedState::default();
        state
            .sync_addresses("EQ-current", &["EQ-old".into()])
            .unwrap();
        assert_eq!(
            state
                .begin_check("EQ-old", false, WalletLivePresence::Unknown)
                .unwrap(),
            None
        );
        let first = state
            .begin_check("EQ-old", false, WalletLivePresence::Ready)
            .unwrap()
            .unwrap();
        assert_eq!(
            state.apply_check(
                "EQ-old",
                first + 1,
                WalletParkedCheckOutcome::Funded { balance_nano: 7 },
            ),
            Err(WalletParkedError::StaleCheck {
                expected: Some(first),
                received: first + 1,
            })
        );
        state
            .apply_check(
                "EQ-old",
                first,
                WalletParkedCheckOutcome::Funded { balance_nano: 7 },
            )
            .unwrap();
        assert!(!state.hidden("EQ-old"));
        assert_eq!(state.aggregate_balance_nano().unwrap(), Some(7));

        let refresh = state
            .begin_check("EQ-old", true, WalletLivePresence::Ready)
            .unwrap()
            .unwrap();
        assert!(!state.hidden("EQ-old"));
        state
            .apply_check("EQ-old", refresh, WalletParkedCheckOutcome::Unknown)
            .unwrap();
        assert_eq!(state.aggregate_balance_nano().unwrap(), None);
    }

    #[test]
    fn empty_wallets_are_hidden_drop_candidates_and_failed_drop_becomes_unknown() {
        let mut state = WalletParkedState::default();
        state
            .sync_addresses("EQ-current", &["EQ-empty".into()])
            .unwrap();
        let serial = state
            .begin_check("EQ-empty", false, WalletLivePresence::Ready)
            .unwrap()
            .unwrap();
        state
            .apply_check("EQ-empty", serial, WalletParkedCheckOutcome::Empty)
            .unwrap();
        assert!(state.hidden("EQ-empty"));
        assert_eq!(
            state.empty_drop_candidates(),
            vec!["EQ-empty".to_string()]
        );
        state.mark_drop_failed("EQ-empty").unwrap();
        assert!(!state.hidden("EQ-empty"));
        assert_eq!(state.aggregate_balance_nano().unwrap(), None);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SendingClockPose {
    pub minute_turns: f64,
    pub hour_turns: f64,
}

pub fn sending_clock_pose(elapsed_ms: i64) -> SendingClockPose {
    const HOUR_TURN_MS: i64 = 2_000;
    const MINUTE_TURNS_PER_HOUR_TURN: f64 = 3.0;
    let progress = elapsed_ms.rem_euclid(HOUR_TURN_MS) as f64 / HOUR_TURN_MS as f64;
    SendingClockPose {
        minute_turns: MINUTE_TURNS_PER_HOUR_TURN * progress,
        hour_turns: 0.25 + progress,
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GlareCycle {
    pub birth_ms: Option<i64>,
    pub death_ms: i64,
}

impl GlareCycle {
    pub fn tick(&mut self, now_ms: i64, duration_ms: i64, pause_ms: i64) -> bool {
        if duration_ms < 0 || pause_ms < 0 {
            return false;
        }
        if now_ms.saturating_sub(self.death_ms) > pause_ms {
            self.birth_ms = Some(now_ms);
            self.death_ms = now_ms.saturating_add(duration_ms);
            true
        } else {
            false
        }
    }

    pub fn progress(&self, now_ms: i64) -> Option<f64> {
        let birth_ms = self.birth_ms?;
        let duration = self.death_ms.saturating_sub(birth_ms);
        if duration <= 0 || now_ms < birth_ms || now_ms > self.death_ms {
            return None;
        }
        Some((now_ms - birth_ms) as f64 / duration as f64)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlareBand {
    pub from: f64,
    pub till: f64,
}

pub fn compute_glare_band(progress: f64, extent: f64, width: f64) -> GlareBand {
    let from = -width + (extent + 2.0 * width) * progress;
    GlareBand {
        from,
        till: from + width,
    }
}

pub fn recovery_seed_from_words(words: &[String]) -> Result<Vec<u8>, WalletRecoveryError> {
    let joined = words.join(" ").into_bytes();
    if joined.len() > WALLET_RECOVERY_SEED_BYTES {
        return Err(WalletRecoveryError::SeedTooLong);
    }
    let mut seed = vec![b' '; WALLET_RECOVERY_SEED_BYTES];
    seed[..joined.len()].copy_from_slice(&joined);
    Ok(seed)
}

pub fn split_recovery_seed(
    seed: &[u8],
    count: usize,
) -> Result<Vec<Vec<u8>>, WalletRecoveryError> {
    if count < 2 || seed.is_empty() {
        return Err(WalletRecoveryError::InvalidShareCount);
    }
    let mut shares = vec![vec![0u8; seed.len()]; count];
    let mut last = seed.to_vec();
    for share in shares.iter_mut().take(count - 1) {
        getrandom::getrandom(share).map_err(|_| WalletRecoveryError::EntropyUnavailable)?;
        for (target, random) in last.iter_mut().zip(share.iter()) {
            *target ^= *random;
        }
    }
    shares[count - 1] = last;
    Ok(shares)
}

pub fn combine_recovery_shares(
    shares: &[Vec<u8>],
) -> Result<Vec<u8>, WalletRecoveryError> {
    let length = shares
        .first()
        .filter(|share| !share.is_empty())
        .map(Vec::len)
        .ok_or(WalletRecoveryError::InvalidShareSet)?;
    if shares.iter().any(|share| share.len() != length) {
        return Err(WalletRecoveryError::MismatchedShareLength);
    }
    let mut result = vec![0u8; length];
    for share in shares {
        for (target, value) in result.iter_mut().zip(share.iter()) {
            *target ^= *value;
        }
    }
    Ok(result)
}

pub struct RecoveryShareKeyPair {
    secret: [u8; WALLET_RECOVERY_PUBLIC_KEY_BYTES],
    public: [u8; WALLET_RECOVERY_PUBLIC_KEY_BYTES],
}

impl RecoveryShareKeyPair {
    pub fn generate() -> Result<Self, WalletRecoveryError> {
        let mut secret = [0u8; WALLET_RECOVERY_PUBLIC_KEY_BYTES];
        getrandom::getrandom(&mut secret).map_err(|_| WalletRecoveryError::EntropyUnavailable)?;
        let private = StaticSecret::from(secret);
        let public = X25519PublicKey::from(&private).to_bytes();
        Ok(Self { secret, public })
    }

    pub fn public_key(&self) -> [u8; WALLET_RECOVERY_PUBLIC_KEY_BYTES] {
        self.public
    }

    pub fn decrypt(
        &self,
        envelope: &EncryptedRecoveryShare,
    ) -> Result<Vec<u8>, WalletRecoveryError> {
        if envelope.ephemeral_public_key.len() != WALLET_RECOVERY_PUBLIC_KEY_BYTES
            || envelope.nonce.len() != WALLET_RECOVERY_NONCE_BYTES
        {
            return Err(WalletRecoveryError::MalformedEnvelope);
        }
        let mut peer_bytes = [0u8; WALLET_RECOVERY_PUBLIC_KEY_BYTES];
        peer_bytes.copy_from_slice(&envelope.ephemeral_public_key);
        let private = StaticSecret::from(self.secret);
        let peer = X25519PublicKey::from(peer_bytes);
        let shared = private.diffie_hellman(&peer);
        let mut key = derive_recovery_share_key(shared.as_bytes())?;
        let aad = recovery_share_aad(&envelope.ephemeral_public_key, &self.public);
        let cipher = XChaCha20Poly1305::new_from_slice(&key)
            .map_err(|_| WalletRecoveryError::EncryptionFailed)?;
        key.zeroize();
        cipher
            .decrypt(
                XNonce::from_slice(&envelope.nonce),
                Payload {
                    msg: &envelope.ciphertext,
                    aad: &aad,
                },
            )
            .map_err(|_| WalletRecoveryError::AuthenticationFailed)
    }
}

impl fmt::Debug for RecoveryShareKeyPair {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RecoveryShareKeyPair")
            .field("public", &self.public)
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

impl Drop for RecoveryShareKeyPair {
    fn drop(&mut self) {
        self.secret.zeroize();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptedRecoveryShare {
    pub ephemeral_public_key: Vec<u8>,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

pub fn encrypt_recovery_share(
    holder_public_key: &[u8],
    share: &[u8],
) -> Result<EncryptedRecoveryShare, WalletRecoveryError> {
    if holder_public_key.len() != WALLET_RECOVERY_PUBLIC_KEY_BYTES || share.is_empty() {
        return Err(WalletRecoveryError::InvalidRecipient);
    }
    let mut holder_bytes = [0u8; WALLET_RECOVERY_PUBLIC_KEY_BYTES];
    holder_bytes.copy_from_slice(holder_public_key);
    let holder = X25519PublicKey::from(holder_bytes);
    let mut ephemeral_secret = [0u8; WALLET_RECOVERY_PUBLIC_KEY_BYTES];
    getrandom::getrandom(&mut ephemeral_secret)
        .map_err(|_| WalletRecoveryError::EntropyUnavailable)?;
    let private = StaticSecret::from(ephemeral_secret);
    ephemeral_secret.zeroize();
    let public = X25519PublicKey::from(&private).to_bytes();
    let shared = private.diffie_hellman(&holder);
    let mut key = derive_recovery_share_key(shared.as_bytes())?;
    let mut nonce = [0u8; WALLET_RECOVERY_NONCE_BYTES];
    getrandom::getrandom(&mut nonce).map_err(|_| WalletRecoveryError::EntropyUnavailable)?;
    let aad = recovery_share_aad(&public, holder_public_key);
    let cipher = XChaCha20Poly1305::new_from_slice(&key)
        .map_err(|_| WalletRecoveryError::EncryptionFailed)?;
    key.zeroize();
    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload { msg: share, aad: &aad },
        )
        .map_err(|_| WalletRecoveryError::EncryptionFailed)?;
    Ok(EncryptedRecoveryShare {
        ephemeral_public_key: public.to_vec(),
        nonce: nonce.to_vec(),
        ciphertext,
    })
}

fn derive_recovery_share_key(
    shared_secret: &[u8; WALLET_RECOVERY_PUBLIC_KEY_BYTES],
) -> Result<[u8; WALLET_KEY_BYTES], WalletRecoveryError> {
    if shared_secret.iter().all(|byte| *byte == 0) {
        return Err(WalletRecoveryError::InvalidRecipient);
    }
    let hkdf = Hkdf::<Sha256>::new(Some(WALLET_RECOVERY_AAD_PREFIX), shared_secret);
    let mut key = [0u8; WALLET_KEY_BYTES];
    hkdf.expand(b"share-wrap", &mut key)
        .map_err(|_| WalletRecoveryError::EncryptionFailed)?;
    Ok(key)
}

fn recovery_share_aad(ephemeral_public_key: &[u8], holder_public_key: &[u8]) -> Vec<u8> {
    let mut aad = Vec::with_capacity(
        WALLET_RECOVERY_AAD_PREFIX.len()
            + ephemeral_public_key.len()
            + holder_public_key.len()
            + 2,
    );
    aad.extend_from_slice(WALLET_RECOVERY_AAD_PREFIX);
    aad.push(0);
    aad.extend_from_slice(ephemeral_public_key);
    aad.push(0);
    aad.extend_from_slice(holder_public_key);
    aad
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WalletRecoveryError {
    #[error("wallet recovery phrase is longer than the fixed seed field")]
    SeedTooLong,
    #[error("wallet recovery share count is invalid")]
    InvalidShareCount,
    #[error("wallet recovery share set is empty or invalid")]
    InvalidShareSet,
    #[error("wallet recovery shares have different lengths")]
    MismatchedShareLength,
    #[error("wallet recovery recipient public key is invalid")]
    InvalidRecipient,
    #[error("wallet recovery encrypted share is malformed")]
    MalformedEnvelope,
    #[error("secure entropy is unavailable")]
    EntropyUnavailable,
    #[error("wallet recovery share encryption failed")]
    EncryptionFailed,
    #[error("wallet recovery share authentication failed")]
    AuthenticationFailed,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WalletError {
    #[error("wallet account id is invalid")]
    InvalidAccountId,
    #[error("wallet account {0:?} already exists")]
    DuplicateAccount(WalletAccountId),
    #[error("wallet account {0:?} was not found")]
    AccountNotFound(WalletAccountId),
    #[error("wallet account {0:?} is frozen")]
    AccountFrozen(WalletAccountId),
    #[error("wallet amount is invalid")]
    InvalidAmount,
    #[error("wallet transfer source and destination must differ")]
    SameAccountTransfer,
    #[error("wallet balance overflow")]
    BalanceOverflow,
    #[error("wallet account {account_id:?} has {available_minor} {currency} minor units but requires {required_minor}")]
    InsufficientFunds {
        account_id: WalletAccountId,
        currency: String,
        available_minor: i64,
        required_minor: i64,
    },
    #[error("wallet ledger entry {0} was not found")]
    EntryNotFound(String),
    #[error("wallet ledger entry {0} is not refundable")]
    NotRefundable(String),
    #[error("wallet request id {0} was already used for a different operation")]
    RequestConflict(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usd(amount_minor: i64) -> Money {
        Money {
            currency: "usd".into(),
            amount_minor,
        }
    }

    #[test]
    fn idempotency_key_conflicts_fail_closed() {
        let buyer = WalletAccountId("wallet:buyer".into());
        let seller = WalletAccountId("wallet:seller".into());
        let other = WalletAccountId("wallet:other".into());
        let mut ledger = WalletLedger::default();
        for (id, owner) in [
            (buyer.clone(), "human:buyer"),
            (seller.clone(), "human:seller"),
            (other.clone(), "human:other"),
        ] {
            ledger.create_account(id, ActorId::new(owner), 1).unwrap();
        }

        ledger
            .credit("credit:conflict", &buyer, usd(1_000), Some("seed".into()), 2)
            .unwrap();
        assert!(matches!(
            ledger.credit("credit:conflict", &buyer, usd(999), Some("seed".into()), 3),
            Err(WalletError::RequestConflict(id)) if id == "credit:conflict"
        ));
        assert!(matches!(
            ledger.credit("credit:conflict", &seller, usd(1_000), Some("seed".into()), 3),
            Err(WalletError::RequestConflict(id)) if id == "credit:conflict"
        ));
        assert!(matches!(
            ledger.credit("credit:conflict", &buyer, usd(1_000), Some("changed".into()), 3),
            Err(WalletError::RequestConflict(id)) if id == "credit:conflict"
        ));
        assert_eq!(ledger.accounts[&buyer].balance("USD"), 1_000);
        assert_eq!(ledger.accounts[&seller].balance("USD"), 0);

        let transfer = ledger
            .transfer(
                "transfer:conflict",
                &buyer,
                &seller,
                usd(250),
                Some("invoice:1".into()),
                4,
            )
            .unwrap();
        let duplicate = ledger
            .transfer(
                "transfer:conflict",
                &buyer,
                &seller,
                usd(250),
                Some("invoice:1".into()),
                5,
            )
            .unwrap();
        assert_eq!(duplicate.id, transfer.id);
        assert!(matches!(
            ledger.transfer(
                "transfer:conflict",
                &buyer,
                &other,
                usd(250),
                Some("invoice:1".into()),
                5,
            ),
            Err(WalletError::RequestConflict(id)) if id == "transfer:conflict"
        ));
        assert!(matches!(
            ledger.transfer(
                "transfer:conflict",
                &buyer,
                &seller,
                usd(251),
                Some("invoice:1".into()),
                5,
            ),
            Err(WalletError::RequestConflict(id)) if id == "transfer:conflict"
        ));
        assert_eq!(ledger.accounts[&buyer].balance("USD"), 750);
        assert_eq!(ledger.accounts[&seller].balance("USD"), 250);
        assert_eq!(ledger.accounts[&other].balance("USD"), 0);

        let refund = ledger
            .refund_transfer("refund:conflict", &transfer.id, 6)
            .unwrap();
        let refund_duplicate = ledger
            .refund_transfer("refund:conflict", &transfer.id, 7)
            .unwrap();
        assert_eq!(refund_duplicate.id, refund.id);
        let unrelated = ledger
            .credit("credit:unrelated", &buyer, usd(10), None, 8)
            .unwrap();
        assert!(matches!(
            ledger.refund_transfer("refund:conflict", &unrelated.id, 9),
            Err(WalletError::RequestConflict(id)) if id == "refund:conflict"
        ));
        assert_eq!(ledger.accounts[&buyer].balance("USD"), 1_010);
        assert_eq!(ledger.accounts[&seller].balance("USD"), 0);
    }

    #[test]
    fn transfer_is_idempotent_and_refundable() {
        let buyer = WalletAccountId("wallet:buyer".into());
        let seller = WalletAccountId("wallet:seller".into());
        let mut ledger = WalletLedger::default();
        ledger
            .create_account(buyer.clone(), ActorId::new("human:buyer"), 1)
            .unwrap();
        ledger
            .create_account(seller.clone(), ActorId::new("human:seller"), 1)
            .unwrap();
        ledger
            .credit("credit:1", &buyer, usd(1_000), None, 2)
            .unwrap();
        let transfer = ledger
            .transfer(
                "pay:1",
                &buyer,
                &seller,
                usd(250),
                Some("invoice:1".into()),
                3,
            )
            .unwrap();
        let duplicate = ledger
            .transfer(
                "pay:1",
                &buyer,
                &seller,
                usd(250),
                Some("invoice:1".into()),
                4,
            )
            .unwrap();
        assert_eq!(duplicate.id, transfer.id);
        assert_eq!(ledger.accounts[&buyer].balance("USD"), 750);
        assert_eq!(ledger.accounts[&seller].balance("USD"), 250);

        let refund = ledger.refund_transfer("refund:1", &transfer.id, 5).unwrap();
        assert_eq!(refund.kind, LedgerEntryKind::Refund);
        assert_eq!(ledger.accounts[&buyer].balance("USD"), 1_000);
        assert_eq!(ledger.accounts[&seller].balance("USD"), 0);
    }
}


#[cfg(test)]
mod wallet_runtime_tests {
    use super::*;
    use std::sync::Mutex;

    struct TestProtectionProvider {
        id: &'static str,
        kind: WalletKeyProtectionKind,
        wrapping_key: [u8; WALLET_KEY_BYTES],
        removed: Mutex<Vec<String>>,
    }

    impl TestProtectionProvider {
        fn new(id: &'static str, kind: WalletKeyProtectionKind, byte: u8) -> Self {
            Self {
                id,
                kind,
                wrapping_key: [byte; WALLET_KEY_BYTES],
                removed: Mutex::new(Vec::new()),
            }
        }
    }

    impl WalletProtectionProvider for TestProtectionProvider {
        fn id(&self) -> &'static str {
            self.id
        }

        fn kind(&self) -> WalletKeyProtectionKind {
            self.kind
        }

        fn enroll(&self) -> Result<WalletProtectionMaterial, WalletProtectionError> {
            Ok(WalletProtectionMaterial {
                reference: format!("handle:{}", self.id),
                wrapping_key: WalletSecretBytes::new(self.wrapping_key.to_vec()),
            })
        }

        fn unwrap(&self, reference: &str) -> Result<WalletSecretBytes, WalletProtectionError> {
            if reference != format!("handle:{}", self.id) {
                return Err(WalletProtectionError::Absent);
            }
            Ok(WalletSecretBytes::new(self.wrapping_key.to_vec()))
        }

        fn remove(&self, reference: &str) -> Result<(), WalletProtectionError> {
            self.removed.lock().unwrap().push(reference.to_string());
            Ok(())
        }
    }

    #[test]
    fn key_protection_fences_stale_epoch_and_never_persists_plaintext_key() {
        let provider =
            TestProtectionProvider::new("system", WalletKeyProtectionKind::System, 0x5a);
        let wallet_key = WalletSecretBytes::new(vec![0x33; WALLET_KEY_BYTES]);
        let mut state = WalletKeyProtectionState::default();

        let epoch = state.install(0, &provider, &wallet_key).unwrap();
        let envelope = state.envelope.as_ref().unwrap();
        assert_eq!(epoch, 1);
        assert_eq!(envelope.provider_id, "system");
        assert!(!envelope.ciphertext.windows(WALLET_KEY_BYTES).any(|window| window == wallet_key.expose()));
        assert!(matches!(
            state.unlock(0, &provider),
            Err(WalletProtectionError::StaleEpoch { current: 1, .. })
        ));

        let unlocked = state.unlock(epoch, &provider).unwrap();
        assert_eq!(unlocked.expose(), wallet_key.expose());
        assert!(!state.locked);
    }

    #[test]
    fn key_protection_rotation_commits_new_envelope_and_retires_old_handle() {
        let first =
            TestProtectionProvider::new("passcode", WalletKeyProtectionKind::Passcode, 0x11);
        let second =
            TestProtectionProvider::new("hardware", WalletKeyProtectionKind::Hardware, 0x22);
        let wallet_key = WalletSecretBytes::new(vec![0x7c; WALLET_KEY_BYTES]);
        let mut state = WalletKeyProtectionState::default();
        let installed = state.install(0, &first, &wallet_key).unwrap();
        let rotated = state.rotate(installed, &first, &second).unwrap();

        assert_eq!(rotated, 2);
        assert_eq!(state.envelope.as_ref().unwrap().provider_id, "hardware");
        assert_eq!(
            second.unwrap(&state.envelope.as_ref().unwrap().provider_reference)
                .unwrap()
                .expose(),
            [0x22; WALLET_KEY_BYTES]
        );
        assert_eq!(
            first.removed.lock().unwrap().as_slice(),
            ["handle:passcode".to_string()]
        );
        let unlocked = state.unlock(rotated, &second).unwrap();
        assert_eq!(unlocked.expose(), wallet_key.expose());
    }

    #[test]
    fn recovery_phrase_split_combine_and_holder_encryption_fail_closed() {
        let words = vec![
            "lotus".to_string(),
            "river".to_string(),
            "moon".to_string(),
        ];
        let seed = recovery_seed_from_words(&words).unwrap();
        assert_eq!(seed.len(), WALLET_RECOVERY_SEED_BYTES);
        assert!(seed.starts_with(b"lotus river moon"));

        let shares = split_recovery_seed(&seed, 3).unwrap();
        assert_eq!(shares.len(), 3);
        assert_eq!(combine_recovery_shares(&shares).unwrap(), seed);
        let mut broken = shares.clone();
        broken[2].pop();
        assert_eq!(
            combine_recovery_shares(&broken),
            Err(WalletRecoveryError::MismatchedShareLength)
        );

        let holder = RecoveryShareKeyPair::generate().unwrap();
        let mut encrypted =
            encrypt_recovery_share(&holder.public_key(), &shares[0]).unwrap();
        assert_eq!(holder.decrypt(&encrypted).unwrap(), shares[0]);
        encrypted.ciphertext[0] ^= 0x40;
        assert_eq!(
            holder.decrypt(&encrypted),
            Err(WalletRecoveryError::AuthenticationFailed)
        );
    }

    #[test]
    fn rates_preserve_selected_currency_and_use_5m_refresh_30s_retry() {
        let mut rates = WalletRateState::default();
        rates.set_currency(" eur ").unwrap();
        rates
            .apply_snapshot(
                [
                    ("USD".to_string(), 1_000_000),
                    ("eur".to_string(), 920_000),
                ],
                1_000,
            )
            .unwrap();
        assert_eq!(rates.selected_currency, "EUR");
        assert_eq!(rates.current_quote_micros(), Some(920_000));
        assert_eq!(rates.refresh_at_ms, 1_000 + WALLET_RATE_REFRESH_MS);
        assert!(!rates.refresh_due(rates.refresh_at_ms - 1));
        assert!(rates.refresh_due(rates.refresh_at_ms));
        assert_eq!(rates.currencies(), vec!["EUR".to_string(), "USD".to_string()]);

        rates.mark_refresh_failure(2_000);
        assert!(rates.retrying);
        assert_eq!(rates.refresh_at_ms, 2_000 + WALLET_RATE_RETRY_MS);
        assert_eq!(
            rates.apply_snapshot(Vec::<(String, i64)>::new(), 3_000),
            Err(WalletRateError::EmptySnapshot)
        );
    }

    #[test]
    fn onramp_replacement_fences_stale_callback_and_optional_base_currency_falls_back() {
        let mut onramp = WalletOnrampState::default();
        let first = onramp
            .begin("wallet-address", "asset", Some("EUR"))
            .unwrap();
        let second = onramp
            .begin("replacement-address", "asset", Some("EUR"))
            .unwrap();
        assert!(matches!(
            onramp.resolve_provider(first, &[]),
            Err(WalletOnrampError::StaleRequest { expected, received })
                if expected == second && received == first
        ));

        let provider = OnrampProviderInfo::new("provider-1", None::<Vec<String>>);
        let spec = onramp.resolve_provider(second, &[provider]).unwrap();
        assert_eq!(spec.provider_id, "provider-1");
        assert_eq!(spec.base_currency, None);
        assert_eq!(
            onramp.complete(second, Some("  https://example.invalid/session  ")).unwrap(),
            Some("https://example.invalid/session".to_string())
        );
        assert_eq!(
            onramp.active.as_ref().unwrap().status,
            WalletOnrampStatus::Completed
        );
    }

    #[test]
    fn panel_state_is_singleton_and_restore_reuses_generation() {
        let mut panel = WalletPanelState::default();
        let first = panel.show();
        assert_eq!(first, 1);
        assert!(panel.minimize());
        let restored = panel.show();
        assert_eq!(restored, first);
        assert!(panel.visible && panel.active && !panel.minimized);
        assert!(panel.close());
        let reopened = panel.show();
        assert_eq!(reopened, 2);
    }

    #[test]
    fn existing_balance_is_once_per_opening_and_failure_retries_only_after_reopen() {
        let mut panel = WalletPanelState::default();
        panel.show();
        let first = panel
            .begin_existing_balance_request(7)
            .unwrap()
            .unwrap();
        panel.fail_existing_balance_request(first.serial).unwrap();
        assert!(panel
            .begin_existing_balance_request(7)
            .unwrap()
            .is_none());

        assert!(panel.close());
        panel.show();
        let second = panel
            .begin_existing_balance_request(7)
            .unwrap()
            .unwrap();
        assert!(second.serial > first.serial);
    }

    #[test]
    fn existing_balance_drops_stale_network_result_and_clears_on_close() {
        let mut panel = WalletPanelState::default();
        panel.show();
        let request = panel
            .begin_existing_balance_request(11)
            .unwrap()
            .unwrap();
        assert!(!panel
            .apply_existing_balance_result(
                12,
                request.serial,
                Some("https://example.invalid/recover".into()),
            )
            .unwrap());
        assert_eq!(panel.existing_balance.url, None);
        assert!(panel
            .begin_existing_balance_request(12)
            .unwrap()
            .is_none());

        assert!(panel.close());
        assert_eq!(panel.existing_balance.url, None);
        assert_eq!(panel.existing_balance.requested_generation, None);
        assert_eq!(panel.existing_balance.in_flight, None);
    }

    #[test]
    fn sending_effect_math_matches_clock_and_glare_contract() {
        assert_eq!(
            sending_clock_pose(500),
            SendingClockPose {
                minute_turns: 0.75,
                hour_turns: 0.5,
            }
        );
        let mut cycle = GlareCycle::default();
        assert!(!cycle.tick(100, 400, 200));
        assert!(cycle.tick(201, 400, 200));
        assert_eq!(cycle.progress(201), Some(0.0));
        assert_eq!(cycle.progress(401), Some(0.5));
        assert_eq!(cycle.progress(602), None);
        assert_eq!(
            compute_glare_band(0.5, 100.0, 20.0),
            GlareBand {
                from: 50.0,
                till: 70.0,
            }
        );
    }

    #[test]
    fn runtime_state_is_durable_inside_existing_wallet_ledger_owner() {
        let mut ledger = WalletLedger::default();
        ledger.runtime.rates.set_currency("JPY").unwrap();
        ledger.runtime.panel.show();
        let encoded = serde_json::to_string(&ledger).unwrap();
        let restored: WalletLedger = serde_json::from_str(&encoded).unwrap();
        assert_eq!(restored.runtime.rates.selected_currency, "JPY");
        assert!(restored.runtime.panel.visible);
    }

    fn sponsored_identity(address: &str, revision: u64) -> WalletTransferIdentity {
        WalletTransferIdentity {
            network: 1,
            address: address.into(),
            public_key: vec![7; 32],
            revision,
        }
    }

    #[test]
    fn sponsored_fee_uses_maximum_minimum_and_requires_fresh_quota_and_relayer() {
        let identity = sponsored_identity("EQ-wallet", 1);
        let mut state = WalletSponsoredFeeState::default();
        state.reset_for_network_generation(4);
        assert_eq!(
            state
                .begin_request(
                    4,
                    identity.clone(),
                    100_000_000,
                    200_000_000,
                    1_000,
                    false,
                )
                .unwrap(),
            Some(1)
        );
        state
            .apply_info(
                1,
                4,
                &identity,
                WalletSponsoredFeeInfo {
                    relayer_address: "EQ-relayer".into(),
                    min_amount_nano: 300_000_000,
                    reset_at_ms: 120_000,
                    left: 2,
                    available: true,
                },
                2_000,
            )
            .unwrap();
        assert_eq!(state.terms.effective_min_nano, 300_000_000);
        assert!(state.terms.fresh);
        assert!(state.terms.usable);
        assert!(state.terms.eligible(300_000_000, "EQ-destination"));
        assert!(!state.terms.eligible(299_999_999, "EQ-destination"));
        assert!(!state.terms.eligible(300_000_000, "EQ-wallet"));

        assert_eq!(
            state
                .begin_request(
                    4,
                    identity.clone(),
                    100_000_000,
                    200_000_000,
                    16_000,
                    true,
                )
                .unwrap(),
            Some(2)
        );
        state
            .apply_info(
                2,
                4,
                &identity,
                WalletSponsoredFeeInfo {
                    relayer_address: "EQ-relayer".into(),
                    min_amount_nano: 350_000_000,
                    reset_at_ms: 120_000,
                    left: 0,
                    available: true,
                },
                16_001,
            )
            .unwrap();
        assert_eq!(state.terms.effective_min_nano, 350_000_000);
        assert!(!state.terms.usable);

        assert_eq!(
            state
                .begin_request(
                    4,
                    identity.clone(),
                    100_000_000,
                    200_000_000,
                    31_000,
                    true,
                )
                .unwrap(),
            Some(3)
        );
        state
            .apply_info(
                3,
                4,
                &identity,
                WalletSponsoredFeeInfo {
                    relayer_address: " bad-relayer ".into(),
                    min_amount_nano: 400_000_000,
                    reset_at_ms: 120_000,
                    left: 1,
                    available: true,
                },
                31_001,
            )
            .unwrap();
        assert!(!state.terms.usable);
    }

    #[test]
    fn sponsored_fee_fences_serial_generation_identity_and_honors_time_windows() {
        let identity = sponsored_identity("EQ-wallet", 1);
        let other = sponsored_identity("EQ-other", 1);
        let mut state = WalletSponsoredFeeState::default();
        state.reset_for_network_generation(9);
        assert_eq!(
            state
                .begin_request(
                    9,
                    identity.clone(),
                    100_000_000,
                    100_000_000,
                    0,
                    false,
                )
                .unwrap(),
            Some(1)
        );
        assert_eq!(
            state
                .begin_request(
                    9,
                    identity.clone(),
                    100_000_000,
                    100_000_000,
                    WALLET_SPONSORED_FEE_REQUEST_TIMEOUT_MS - 1,
                    true,
                )
                .unwrap(),
            None
        );
        state
            .sync_context(
                9,
                identity.clone(),
                100_000_000,
                100_000_000,
                WALLET_SPONSORED_FEE_REQUEST_TIMEOUT_MS,
            )
            .unwrap();
        assert!(state.active_request.is_none());
        assert!(state.last_failure_at_ms.is_some());
        assert!(state.request_due(WALLET_SPONSORED_FEE_REQUEST_TIMEOUT_MS));
        assert_eq!(
            state
                .begin_request(
                    9,
                    identity.clone(),
                    100_000_000,
                    100_000_000,
                    WALLET_SPONSORED_FEE_REQUEST_TIMEOUT_MS,
                    false,
                )
                .unwrap(),
            Some(2)
        );
        assert_eq!(
            state.apply_info(
                1,
                9,
                &identity,
                WalletSponsoredFeeInfo {
                    relayer_address: "EQ-relayer".into(),
                    min_amount_nano: 100_000_000,
                    reset_at_ms: 200_000,
                    left: 1,
                    available: true,
                },
                WALLET_SPONSORED_FEE_REQUEST_TIMEOUT_MS + 1,
            ),
            Err(WalletSponsoredFeeError::StaleRequest)
        );
        assert_eq!(
            state.apply_info(
                2,
                9,
                &other,
                WalletSponsoredFeeInfo {
                    relayer_address: "EQ-relayer".into(),
                    min_amount_nano: 100_000_000,
                    reset_at_ms: 200_000,
                    left: 1,
                    available: true,
                },
                WALLET_SPONSORED_FEE_REQUEST_TIMEOUT_MS + 1,
            ),
            Err(WalletSponsoredFeeError::StaleRequest)
        );
        let answered_at = WALLET_SPONSORED_FEE_REQUEST_TIMEOUT_MS + 1;
        state
            .apply_info(
                2,
                9,
                &identity,
                WalletSponsoredFeeInfo {
                    relayer_address: "EQ-relayer".into(),
                    min_amount_nano: 100_000_000,
                    reset_at_ms: 200_000,
                    left: 1,
                    available: true,
                },
                answered_at,
            )
            .unwrap();
        let refresh_at =
            answered_at + WALLET_SPONSORED_FEE_REFRESH_MS - WALLET_SPONSORED_FEE_REFRESH_AHEAD_MS;
        assert!(!state.request_due(refresh_at - 1));
        assert!(state.request_due(refresh_at));

        assert_eq!(
            state.sync_context(
                10,
                identity,
                100_000_000,
                100_000_000,
                refresh_at,
            ),
            Err(WalletSponsoredFeeError::StaleNetworkGeneration {
                current: 9,
                received: 10,
            })
        );
    }

    #[test]
    fn sponsored_fee_network_authority_is_runtime_only() {
        let identity = sponsored_identity("EQ-wallet", 3);
        let mut ledger = WalletLedger::default();
        ledger.runtime.sponsored_fees.reset_for_network_generation(2);
        let serial = ledger
            .runtime
            .sponsored_fees
            .begin_request(
                2,
                identity.clone(),
                100_000_000,
                100_000_000,
                1_000,
                false,
            )
            .unwrap()
            .unwrap();
        ledger
            .runtime
            .sponsored_fees
            .apply_info(
                serial,
                2,
                &identity,
                WalletSponsoredFeeInfo {
                    relayer_address: "EQ-relayer".into(),
                    min_amount_nano: 100_000_000,
                    reset_at_ms: 70_000,
                    left: 3,
                    available: true,
                },
                2_000,
            )
            .unwrap();
        assert!(ledger.runtime.sponsored_fees.terms.usable);

        let encoded = serde_json::to_string(&ledger).unwrap();
        assert!(!encoded.contains("sponsoredFees"));
        let restored: WalletLedger = serde_json::from_str(&encoded).unwrap();
        assert_eq!(
            restored.runtime.sponsored_fees,
            WalletSponsoredFeeState::default()
        );
    }

    #[test]
    fn sponsored_quote_expires_with_terms_and_only_paired_send_spends_full_balance() {
        let identity = sponsored_identity("EQ-wallet", 8);
        let mut runtime = WalletRuntimeState::default();
        runtime.sponsored_fees.reset_for_network_generation(3);
        let serial = runtime
            .sponsored_fees
            .begin_request(
                3,
                identity.clone(),
                100_000_000,
                100_000_000,
                1_000,
                false,
            )
            .unwrap()
            .unwrap();
        runtime
            .sponsored_fees
            .apply_info(
                serial,
                3,
                &identity,
                WalletSponsoredFeeInfo {
                    relayer_address: "EQ-relayer".into(),
                    min_amount_nano: 100_000_000,
                    reset_at_ms: 90_000,
                    left: 1,
                    available: true,
                },
                2_000,
            )
            .unwrap();

        let paired = runtime
            .sponsored_fees
            .quote_transfer(
                3,
                identity.clone(),
                100_000_000,
                100_000_000,
                500_000_000,
                "EQ-destination".into(),
                25_000_000,
                2_001,
            )
            .unwrap();
        assert!(paired.paired);
        runtime
            .sponsored_fees
            .authorize_quote(&paired, 500_000_000, 2_002)
            .unwrap();

        let normal = runtime
            .sponsored_fees
            .quote_transfer(
                3,
                identity.clone(),
                100_000_000,
                100_000_000,
                500_000_000,
                "EQ-wallet".into(),
                25_000_000,
                2_003,
            )
            .unwrap();
        assert!(!normal.paired);
        assert_eq!(
            runtime
                .sponsored_fees
                .authorize_quote(&normal, 500_000_000, 2_004),
            Err(WalletTransferQuoteError::InsufficientFees)
        );

        let refresh_serial = runtime
            .sponsored_fees
            .begin_request(
                3,
                identity.clone(),
                100_000_000,
                100_000_000,
                17_001,
                true,
            )
            .unwrap()
            .unwrap();
        runtime
            .sponsored_fees
            .apply_info(
                refresh_serial,
                3,
                &identity,
                WalletSponsoredFeeInfo {
                    relayer_address: "EQ-relayer".into(),
                    min_amount_nano: 200_000_000,
                    reset_at_ms: 90_000,
                    left: 1,
                    available: true,
                },
                17_002,
            )
            .unwrap();
        assert_eq!(
            runtime
                .sponsored_fees
                .authorize_quote(&paired, 500_000_000, 17_003),
            Err(WalletTransferQuoteError::QuoteExpired)
        );
    }

    #[test]
    fn quoted_transfer_reuses_durable_outbound_journal_and_rejects_pairing_drift() {
        let identity = sponsored_identity("EQ-wallet", 2);
        let mut runtime = WalletRuntimeState::default();
        runtime.sponsored_fees.reset_for_network_generation(5);
        let serial = runtime
            .sponsored_fees
            .begin_request(
                5,
                identity.clone(),
                100_000_000,
                100_000_000,
                1_000,
                false,
            )
            .unwrap()
            .unwrap();
        runtime
            .sponsored_fees
            .apply_info(
                serial,
                5,
                &identity,
                WalletSponsoredFeeInfo {
                    relayer_address: "EQ-relayer".into(),
                    min_amount_nano: 100_000_000,
                    reset_at_ms: 100_000,
                    left: 2,
                    available: true,
                },
                2_000,
            )
            .unwrap();
        let quote = runtime
            .sponsored_fees
            .quote_transfer(
                5,
                identity.clone(),
                100_000_000,
                100_000_000,
                400_000_000,
                "EQ-destination".into(),
                20_000_000,
                2_001,
            )
            .unwrap();
        let mut record = OutboundTransferRecord {
            record_id: "quoted-record".into(),
            network: identity.network,
            address: identity.address.clone(),
            public_key: identity.public_key.clone(),
            operation_id: "quoted-operation".into(),
            destination: quote.destination.clone(),
            comment: String::new(),
            collectible: None,
            recipient_actor_id: None,
            served: None,
            amount_nano: quote.amount_nano,
            posted_at_ms: 2_001,
            handoff: OutboundTransferHandoff::Preparation,
            terminal: OutboundTransferTerminal::None,
            message_token: None,
            confirmed_hash: None,
            lookup_attempts: 0,
            lookup_stopped: false,
            paired: true,
            bounce: false,
        };
        record.paired = false;
        assert_eq!(
            runtime.journal_quoted_outbound_transfer(record.clone(), &quote, 400_000_000, 2_002),
            Err(WalletTransferQuoteError::InvalidTransfer)
        );
        record.paired = true;
        runtime
            .journal_quoted_outbound_transfer(record, &quote, 400_000_000, 2_003)
            .unwrap();
        assert_eq!(runtime.outbound_transfers.records.len(), 1);
        assert!(runtime.outbound_transfers.records[0].paired);
    }

    #[test]
    fn live_state_fences_generations_and_requires_two_failures_before_unreachable() {
        let mut live = WalletLiveState::default();
        let first = live.begin_generation().unwrap();
        assert_eq!(first, 1);
        assert!(live.state_refresh_due(0));
        live.apply_presence(first, WalletLivePresence::Ready, 1_000)
            .unwrap();
        assert!(!live.state_unreachable);
        assert!(!live.state_refresh_due(1_000 + WALLET_LIVE_STATE_REFRESH_MS - 1));

        live.note_state_failure(first, 2_000).unwrap();
        assert!(!live.state_unreachable);
        live.note_state_failure(first, 3_000).unwrap();
        assert!(live.state_unreachable);

        let second = live.begin_generation().unwrap();
        assert_eq!(second, 2);
        assert_eq!(live.presence, WalletLivePresence::Unknown);
        assert!(!live.state_unreachable);
        assert_eq!(
            live.note_state_failure(first, 4_000),
            Err(WalletLiveError::StaleGeneration {
                current: second,
                received: first,
            })
        );
    }

    #[test]
    fn live_hidden_history_walk_spends_before_request_and_is_rearmable() {
        let mut live = WalletLiveState::default();
        let generation = live.begin_generation().unwrap();
        for _ in 0..WALLET_LIVE_MAX_HIDDEN_PAGES {
            live.spend_history_page_request(generation).unwrap();
            live.note_history_progress(generation, 0).unwrap();
        }
        assert_eq!(
            live.spend_history_page_request(generation),
            Err(WalletLiveError::HistoryPageBudgetExhausted)
        );
        live.note_history_progress(generation, 1).unwrap();
        assert_eq!(live.hidden_pages_without_visible_rows, 0);
        for _ in 0..3 {
            live.spend_history_page_request(generation).unwrap();
            live.note_history_progress(generation, 0).unwrap();
        }
        live.rearm_history_walk(generation).unwrap();
        assert_eq!(live.hidden_pages_without_visible_rows, 0);
    }

    #[test]
    fn live_stream_fences_epochs_backoffs_and_resets_after_stable_live() {
        let mut stream = WalletStreamState::default();
        let acquire = stream.start("EQ-wallet", 0).unwrap();
        let socket = stream.apply_url(acquire, 10, 120_000).unwrap();
        assert_eq!(stream.phase, WalletStreamPhase::Connecting);
        // tdesktop scheduleRenew converts the relative expiry to a lifetime and
        // renews at clamp(lifetime - 60s, lifetime / 2, 30min). For a 120s
        // lifetime acquired at t=10ms, the authoritative renew deadline is 60_010ms.
        assert_eq!(stream.renew_at_ms, Some(60_010));
        stream.connected(socket, 20).unwrap();
        stream.note_activity(socket, 25).unwrap();
        assert_eq!(
            stream.keepalive_action(socket, 25 + WALLET_STREAM_STALL_MS + 1).unwrap(),
            WalletStreamKeepalive::Fail
        );
        let retry_generation = stream.fail(socket, 30).unwrap();
        assert_eq!(stream.retry_at_ms, Some(30 + WALLET_STREAM_RETRY_MS[0]));
        assert!(!stream
            .resume_after_backoff(retry_generation, 100)
            .unwrap());
        assert!(stream
            .resume_after_backoff(retry_generation, 530)
            .unwrap());

        let socket = stream.apply_url(retry_generation, 540, 120_000).unwrap();
        stream.connected(socket, 550).unwrap();
        let retry_generation = stream
            .fail(socket, 550 + WALLET_STREAM_STABLE_MS + 1)
            .unwrap();
        assert_eq!(
            stream.retry_at_ms,
            Some(550 + WALLET_STREAM_STABLE_MS + 1 + WALLET_STREAM_RETRY_MS[0])
        );
        assert_eq!(
            stream.note_activity(socket, 1_000),
            Err(WalletLiveError::StaleStreamGeneration {
                current: retry_generation,
                received: socket,
            })
        );
    }

    #[test]
    fn live_stream_coalesces_refresh_and_rechecks_transaction_history_twice() {
        let mut stream = WalletStreamState::default();
        let acquire = stream.start("EQ-wallet", 0).unwrap();
        let socket = stream.apply_url(acquire, 10, 120_000).unwrap();
        stream.connected(socket, 20).unwrap();
        stream.note_transaction_event(socket, 100).unwrap();
        stream
            .want_refresh(
                socket,
                110,
                WalletStreamRefresh {
                    state: false,
                    history: true,
                    collectibles: false,
                },
            )
            .unwrap();
        assert!(stream.take_refresh_if_due(socket, 349).unwrap().is_none());
        let wanted = stream.take_refresh_if_due(socket, 350).unwrap().unwrap();
        assert!(wanted.state && wanted.history && wanted.collectibles);
        assert!(!stream
            .take_history_recheck_if_due(socket, 3_099)
            .unwrap());
        assert!(stream
            .take_history_recheck_if_due(socket, 3_100)
            .unwrap());
        assert_eq!(stream.history_recheck_due_ms, Some(10_100));
        assert!(stream
            .take_history_recheck_if_due(socket, 10_100)
            .unwrap());
        assert_eq!(stream.history_recheck_due_ms, None);
    }

    #[test]
    fn live_state_is_not_restored_as_stale_network_authority() {
        let mut ledger = WalletLedger::default();
        let generation = ledger.runtime.live.begin_generation().unwrap();
        ledger
            .runtime
            .live
            .apply_presence(generation, WalletLivePresence::Ready, 9_000)
            .unwrap();
        ledger.runtime.live.mark_stream_resync(generation, 9_000).unwrap();
        let encoded = serde_json::to_string(&ledger).unwrap();
        assert!(!encoded.contains("\"live\""));
        let restored: WalletLedger = serde_json::from_str(&encoded).unwrap();
        assert_eq!(restored.runtime.live, WalletLiveState::default());
        assert!(restored.runtime.live.state_refresh_due(9_001));
        assert!(restored.runtime.live.stream_resync_due(9_001));
    }
}


#[cfg(test)]
mod outbound_transfer_journal_tests {
    use super::*;

    fn prepared(id: &str, operation: &str) -> OutboundTransferRecord {
        OutboundTransferRecord {
            record_id: id.into(),
            network: 1,
            address: "source-address".into(),
            public_key: vec![7; 32],
            operation_id: operation.into(),
            destination: "destination-address".into(),
            comment: "hello".into(),
            collectible: None,
            recipient_actor_id: None,
            served: None,
            amount_nano: 100_000_000,
            posted_at_ms: 100,
            handoff: OutboundTransferHandoff::Preparation,
            terminal: OutboundTransferTerminal::None,
            message_token: None,
            confirmed_hash: None,
            lookup_attempts: 0,
            lookup_stopped: false,
            paired: false,
            bounce: false,
        }
    }

    #[test]
    fn handoff_is_committed_before_submission_unknown_state_exists() {
        let mut journal = OutboundTransferJournal::default();
        journal.prepare(prepared("record-1", "operation-1")).unwrap();
        assert!(journal.submission_unknown().is_empty());

        journal
            .mark_handoff_possible("record-1", vec![0x42; 32])
            .unwrap();
        assert_eq!(journal.submission_unknown().len(), 1);
        assert_eq!(journal.note_lookup_attempt("record-1").unwrap(), 1);
        assert_eq!(journal.submission_unknown()[0].lookup_attempts, 1);
    }

    #[test]
    fn journal_identity_is_exactly_once_across_record_ids() {
        let mut journal = OutboundTransferJournal::default();
        journal.prepare(prepared("record-a", "operation-a")).unwrap();
        let mut duplicate = prepared("record-b", "operation-a");
        assert_eq!(
            journal.prepare(duplicate.clone()),
            Err(OutboundTransferError::DuplicateIdentity)
        );
        duplicate.operation_id = "operation-b".into();
        journal.prepare(duplicate).unwrap();
        assert_eq!(journal.records.len(), 2);
    }

    #[test]
    fn served_projection_is_durable_bounded_and_only_attaches_after_handoff() {
        let mut journal = OutboundTransferJournal::default();
        journal.prepare(prepared("record-1", "operation-1")).unwrap();
        let projection = OutboundTransferProjection {
            id: "server-message-1".into(),
            counterparty: "EQ-peer".into(),
            counterparty_name: "Peer".into(),
            comment: String::new(),
            collectible: String::new(),
            counterparty_actor_id: None,
            amount_nano: 100_000_000,
            fee_nano: Some(1_000),
            date_ms: Some(1_000),
            peer_transfer: false,
            failed: false,
            comment_encrypted: true,
            gasless: true,
            counterparty_bounceable: false,
        };
        assert_eq!(
            journal.record_served_projection("record-1", projection.clone()),
            Err(OutboundTransferError::InvalidTransition)
        );
        journal
            .mark_handoff_possible("record-1", vec![0x33; 32])
            .unwrap();
        journal
            .record_served_projection("record-1", projection.clone())
            .unwrap();
        assert_eq!(journal.records[0].served.as_ref(), Some(&projection));

        let encoded = serde_json::to_string(&journal).unwrap();
        let restored: OutboundTransferJournal = serde_json::from_str(&encoded).unwrap();
        assert_eq!(restored.records[0].served.as_ref(), Some(&projection));

        let mut invalid = projection;
        invalid.comment_encrypted = true;
        invalid.comment = "private plaintext must not persist".into();
        assert_eq!(
            journal.record_served_projection("record-1", invalid),
            Err(OutboundTransferError::InvalidProjection)
        );
    }

    #[test]
    fn confirmed_terminal_requires_exact_hash_and_never_resubmits() {
        let mut journal = OutboundTransferJournal::default();
        journal.prepare(prepared("record-1", "operation-1")).unwrap();
        journal
            .mark_handoff_possible("record-1", vec![0x11; 32])
            .unwrap();
        assert_eq!(
            journal.settle(
                "record-1",
                OutboundTransferTerminal::Confirmed,
                Some(vec![9; 31]),
            ),
            Err(OutboundTransferError::InvalidConfirmation)
        );
        journal
            .settle(
                "record-1",
                OutboundTransferTerminal::Confirmed,
                Some(vec![9; 32]),
            )
            .unwrap();
        assert!(journal.submission_unknown().is_empty());
        assert_eq!(
            journal.mark_handoff_possible("record-1", vec![1; 32]),
            Err(OutboundTransferError::InvalidTransition)
        );
    }

    #[test]
    fn journal_is_durable_inside_wallet_runtime() {
        let mut runtime = WalletRuntimeState::default();
        runtime
            .outbound_transfers
            .prepare(prepared("record-1", "operation-1"))
            .unwrap();
        runtime
            .outbound_transfers
            .mark_handoff_possible("record-1", vec![3; 32])
            .unwrap();
        let encoded = serde_json::to_string(&runtime).unwrap();
        let restored: WalletRuntimeState = serde_json::from_str(&encoded).unwrap();
        assert_eq!(restored.outbound_transfers.submission_unknown().len(), 1);
    }
}


#[cfg(test)]
mod wallet_address_directory_tests {
    use super::*;

    #[test]
    fn stale_address_reply_never_overwrites_newer_answer() {
        let mut directory = WalletAddressDirectory::default();
        let actor = ActorId("person-1".into());
        assert!(directory
            .apply_user_answer(
                actor.clone(),
                2,
                Some("EQ-current-address".into()),
                vec![2; 32],
            )
            .unwrap());
        assert!(!directory
            .apply_user_answer(
                actor.clone(),
                1,
                Some("EQ-stale-address".into()),
                vec![1; 32],
            )
            .unwrap());
        assert_eq!(directory.known(&actor).address, "EQ-current-address");
        assert_eq!(directory.known(&actor).public_key, vec![2; 32]);
    }

    #[test]
    fn unavailable_service_does_not_fabricate_absent_user_answers() {
        let mut directory = WalletAddressDirectory::default();
        let actor = ActorId("person-2".into());
        directory.set_service_unavailable(true);
        assert_eq!(
            directory.known(&actor).knowledge,
            WalletAddressKnowledge::Unknown
        );
        directory
            .apply_user_answer(actor.clone(), 1, None, Vec::new())
            .unwrap();
        assert_eq!(
            directory.known(&actor).knowledge,
            WalletAddressKnowledge::Absent
        );
    }

    #[test]
    fn owner_reply_can_supply_public_key_for_undeployed_recipient() {
        let mut directory = WalletAddressDirectory::default();
        directory
            .apply_owner_answer(
                "EQ-recipient".into(),
                None,
                vec![9; 32],
            )
            .unwrap();
        let owner = directory.owner("EQ-recipient").unwrap();
        assert_eq!(owner.actor_id, None);
        assert_eq!(owner.public_key, vec![9; 32]);
    }
}

#[cfg(test)]
mod ton_connect_emulation_source_tests {
    use super::*;

    fn action(kind: &str, succeeded: bool, details_json: &str) -> WalletEmulationSourceAction {
        WalletEmulationSourceAction {
            kind: kind.to_string(),
            succeeded,
            details_json: details_json.to_string(),
        }
    }

    #[test]
    fn emulation_projects_actions_sides_excess_and_overflow_safe_net() {
        let source = WalletEmulationSource {
            trace_succeeded: true,
            is_incomplete: false,
            actions: vec![
                action(
                    "ton_transfer",
                    true,
                    r#"{"source":"EQ-own","destination":"EQ-dest","value":"10"}"#,
                ),
                action(
                    "extra_currency_transfer",
                    true,
                    r#"{"source":"EQ-peer","destination":"EQ-own","value":"3"}"#,
                ),
                action(
                    "call_contract",
                    true,
                    r#"{"source":"EQ-peer","destination":"EQ-own","value":"2","opcode":"0xd53276db"}"#,
                ),
                action(
                    "contract_deploy",
                    true,
                    r#"{"destination":"EQ-contract"}"#,
                ),
                action("unknown", false, r#"{}"#),
            ],
        };
        let parsed = parse_wallet_emulation(&source, "EQ-own", 10);
        assert_eq!(parsed.status, WalletEmulationStatus::Shown);
        assert_eq!(parsed.net_nano, -5);
        assert_eq!(parsed.actions.len(), 4);
        assert_eq!(parsed.actions[0].kind, WalletEmulationActionKind::Withdraw);
        assert_eq!(parsed.actions[0].side, WalletEmulationActionSide::Outgoing);
        assert_eq!(parsed.actions[1].kind, WalletEmulationActionKind::Deposit);
        assert_eq!(parsed.actions[1].side, WalletEmulationActionSide::Incoming);
        assert_eq!(parsed.actions[2].kind, WalletEmulationActionKind::Excess);
        assert_eq!(parsed.actions[3].kind, WalletEmulationActionKind::DeployContract);

        let overflow = WalletEmulationSource {
            trace_succeeded: true,
            is_incomplete: false,
            actions: vec![action(
                "ton_transfer",
                true,
                r#"{"source":"EQ-peer","destination":"EQ-own","value":"1"}"#,
            )],
        };
        assert_eq!(
            parse_wallet_emulation(&overflow, "EQ-own", i64::MIN).status,
            WalletEmulationStatus::Empty
        );
    }

    #[test]
    fn emulation_distinguishes_failed_aborted_incomplete_and_empty() {
        let failed = WalletEmulationSource {
            trace_succeeded: false,
            is_incomplete: false,
            actions: vec![action("call_contract", false, r#"{}"#)],
        };
        assert_eq!(
            parse_wallet_emulation(&failed, "EQ-own", 0).status,
            WalletEmulationStatus::Failed
        );

        let aborted = WalletEmulationSource {
            trace_succeeded: false,
            is_incomplete: false,
            actions: vec![action("unknown", false, r#"{}"#)],
        };
        assert_eq!(
            parse_wallet_emulation(&aborted, "EQ-own", 0).status,
            WalletEmulationStatus::Aborted
        );

        let incomplete = WalletEmulationSource {
            trace_succeeded: true,
            is_incomplete: true,
            actions: vec![action("call_contract", true, r#"{}"#)],
        };
        assert_eq!(
            parse_wallet_emulation(&incomplete, "EQ-own", 0).status,
            WalletEmulationStatus::Incomplete
        );

        let shown_without_actions = WalletEmulationSource {
            trace_succeeded: true,
            is_incomplete: false,
            actions: vec![action("unknown", true, r#"{}"#)],
        };
        assert_eq!(
            parse_wallet_emulation(&shown_without_actions, "EQ-own", 0).status,
            WalletEmulationStatus::Empty
        );
        assert_eq!(
            parse_wallet_emulation(&shown_without_actions, "", 0).status,
            WalletEmulationStatus::Empty
        );
    }

    #[test]
    fn emulation_opcode_accepts_signed_numeric_bits_and_rejects_malformed_amounts() {
        let negative_excess = -(0x2acd8925_i64);
        let source = WalletEmulationSource {
            trace_succeeded: true,
            is_incomplete: false,
            actions: vec![
                action(
                    "call_contract",
                    true,
                    &format!(
                        r#"{{"source":"EQ-peer","destination":"EQ-own","value":"12x","opcode":{negative_excess}}}"#
                    ),
                ),
                action(
                    "other",
                    true,
                    r#"{"source":"EQ-peer","destination":"EQ-other","opcode":"0x123456789"}"#,
                ),
            ],
        };
        let parsed = parse_wallet_emulation(&source, "EQ-own", 0);
        assert_eq!(parsed.actions[0].kind, WalletEmulationActionKind::Excess);
        assert_eq!(parsed.actions[0].amount_nano, None);
        assert_eq!(parsed.actions[1].kind, WalletEmulationActionKind::Unknown);
    }
}

