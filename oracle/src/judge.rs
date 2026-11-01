//! Oracle: judge every vector with Agave's own code.
//!
//! Three layers, mirroring what a validator does before a transaction can be
//! included in a block:
//!   1. `agave-transaction-view`  : the packet parser used by the banking stage
//!      (`SanitizedTransactionView::try_new_sanitized`).
//!   2. `solana-runtime-transaction` : protocol limits (size, 12 sigs / 64 accounts
//!      for v1, 64 instructions, heap bounds) and duplicate-account rejection.
//!   3. `solana-transaction` (SDK) : `VersionedTransaction` wincode decode + `sanitize()`,
//!      plus re-serialization, message bytes and message hash.
//!
//! `valid` is true only when all layers accept. The per-layer results are kept so
//! that a divergence between Agave's own layers is visible too.

use agave_transaction_view::{
    resolved_transaction_view::ResolvedTransactionView, transaction_version::TransactionVersion,
    transaction_view::{SanitizedTransactionView, UnsanitizedTransactionView},
};
use serde::{Deserialize, Serialize};
use solana_message::{v0::LoadedAddresses, VersionedMessage};
use solana_pubkey::Pubkey;
use solana_runtime_transaction::runtime_transaction::RuntimeTransaction;
use solana_transaction::{sanitized::MessageHash, versioned::VersionedTransaction};
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ConfigOut {
    pub priority_fee_lamports: Option<u64>,
    pub compute_unit_limit: Option<u32>,
    pub loaded_accounts_data_size_limit: Option<u32>,
    pub heap_size: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IxOut {
    pub program_id_index: u8,
    pub accounts: Vec<u8>,
    pub data_hex: String,
}

/// Canonical parsed view. Harnesses in other languages emit the same shape.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Fields {
    pub version: String, // "legacy" | "0" | "1"
    pub num_signatures: usize,
    pub header: [u8; 3],
    pub account_keys: Vec<String>, // base58
    pub recent_blockhash: String,  // base58
    pub instructions: Vec<IxOut>,
    pub num_address_table_lookups: usize,
    pub config: Option<ConfigOut>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Verdict {
    pub id: String,
    pub class: String,
    pub expect_valid: bool,
    /// Final answer: would Agave accept this transaction's format?
    pub valid: bool,
    pub view_parse: Result<(), String>,
    pub view_sanitize: Result<(), String>,
    pub runtime_static: Result<(), String>,
    pub runtime_resolved: Result<(), String>,
    pub sdk_decode: Result<(), String>,
    pub sdk_sanitize: Result<(), String>,
    pub fields: Option<Fields>,
    /// SDK re-serialization of the decoded transaction.
    pub sdk_reserialized_hex: Option<String>,
    pub sdk_roundtrip_identical: Option<bool>,
    /// Bytes that are signed (message data) and their hash, from the transaction view.
    pub message_hex: Option<String>,
    pub message_hash: Option<String>,
    pub tx_len: usize,
}

fn key_from_hex(h: &str) -> Pubkey {
    let v = hex::decode(h).expect("hex");
    Pubkey::new_from_array(v.try_into().expect("32 bytes"))
}

pub fn judge(v: &crate::wire::Vector) -> Verdict {
    let bytes = hex::decode(&v.tx_hex).expect("tx hex");
    let mut out = Verdict {
        id: v.id.clone(),
        class: v.class.clone(),
        expect_valid: v.expect_valid,
        valid: false,
        view_parse: Err("not run".into()),
        view_sanitize: Err("not run".into()),
        runtime_static: Err("not run".into()),
        runtime_resolved: Err("not run".into()),
        sdk_decode: Err("not run".into()),
        sdk_sanitize: Err("not run".into()),
        fields: None,
        sdk_reserialized_hex: None,
        sdk_roundtrip_identical: None,
        message_hex: None,
        message_hash: None,
        tx_len: bytes.len(),
    };

    // ---- layer 1: transaction view
    let unsanitized = UnsanitizedTransactionView::try_new_unsanitized(&bytes[..]);
    match unsanitized {
        Err(e) => {
            out.view_parse = Err(format!("{e:?}"));
            out.view_sanitize = Err("parse failed".into());
        }
        Ok(u) => {
            out.view_parse = Ok(());
            match u.sanitize() {
                Err(e) => out.view_sanitize = Err(format!("{e:?}")),
                Ok(view) => {
                    out.view_sanitize = Ok(());
                    out.message_hex = Some(hex::encode(view.message_data()));
                    out.message_hash =
                        Some(VersionedMessage::hash_raw_message(view.message_data()).to_string());
                    out.fields = Some(fields_from_view(&view));
                    // ---- layer 2: runtime limits
                    let is_v0 = matches!(view.version(), TransactionVersion::V0);
                    match RuntimeTransaction::<SanitizedTransactionView<&[u8]>>::try_new(
                        view,
                        MessageHash::Compute,
                        None,
                    ) {
                        Err(e) => {
                            out.runtime_static = Err(format!("{e:?}"));
                            out.runtime_resolved = Err("static failed".into());
                        }
                        Ok(static_tx) => {
                            out.runtime_static = Ok(());
                            let loaded = if is_v0 {
                                Some(LoadedAddresses {
                                    writable: v.loaded_writable_hex.iter().map(|h| key_from_hex(h)).collect(),
                                    readonly: v.loaded_readonly_hex.iter().map(|h| key_from_hex(h)).collect(),
                                })
                            } else {
                                None
                            };
                            match RuntimeTransaction::<ResolvedTransactionView<&[u8]>>::try_new(
                                static_tx,
                                loaded,
                                &HashSet::new(),
                            ) {
                                Err(e) => out.runtime_resolved = Err(format!("{e:?}")),
                                Ok(_) => out.runtime_resolved = Ok(()),
                            }
                        }
                    }
                }
            }
        }
    }

    // ---- layer 3: SDK
    match wincode::deserialize::<VersionedTransaction>(&bytes) {
        Err(e) => {
            out.sdk_decode = Err(format!("{e:?}"));
            out.sdk_sanitize = Err("decode failed".into());
        }
        Ok(tx) => {
            out.sdk_decode = Ok(());
            out.sdk_sanitize = tx.sanitize().map_err(|e| format!("{e:?}"));
            if let Ok(re) = wincode::serialize(&tx) {
                out.sdk_roundtrip_identical = Some(re == bytes);
                out.sdk_reserialized_hex = Some(hex::encode(re));
            }
            if out.fields.is_none() && out.sdk_sanitize.is_ok() {
                out.fields = Some(fields_from_sdk(&tx));
            }
        }
    }

    out.valid = out.view_parse.is_ok()
        && out.view_sanitize.is_ok()
        && out.runtime_static.is_ok()
        && out.runtime_resolved.is_ok()
        && out.sdk_decode.is_ok()
        && out.sdk_sanitize.is_ok();
    out
}

fn fields_from_view(view: &SanitizedTransactionView<&[u8]>) -> Fields {
    let config = view.transaction_config().map(|c| ConfigOut {
        priority_fee_lamports: c.priority_fee_lamports(),
        compute_unit_limit: c.compute_unit_limit(),
        loaded_accounts_data_size_limit: c.loaded_accounts_data_size_limit(),
        heap_size: c.requested_heap_size(),
    });
    Fields {
        version: match view.version() {
            TransactionVersion::Legacy => "legacy".into(),
            TransactionVersion::V0 => "0".into(),
            TransactionVersion::V1 => "1".into(),
        },
        num_signatures: view.signatures().len(),
        header: [
            view.num_required_signatures(),
            view.num_readonly_signed_static_accounts(),
            view.num_readonly_unsigned_static_accounts(),
        ],
        account_keys: view.static_account_keys().iter().map(|k| k.to_string()).collect(),
        recent_blockhash: view.recent_blockhash().to_string(),
        instructions: view
            .instructions_iter()
            .map(|ix| IxOut {
                program_id_index: ix.program_id_index,
                accounts: ix.accounts.to_vec(),
                data_hex: hex::encode(ix.data),
            })
            .collect(),
        num_address_table_lookups: view.num_address_table_lookups() as usize,
        config,
    }
}

fn fields_from_sdk(tx: &VersionedTransaction) -> Fields {
    let m = &tx.message;
    let config = match m {
        VersionedMessage::V1(v1) => Some(ConfigOut {
            priority_fee_lamports: v1.config.priority_fee,
            compute_unit_limit: v1.config.compute_unit_limit,
            loaded_accounts_data_size_limit: v1.config.loaded_accounts_data_size_limit,
            heap_size: v1.config.heap_size,
        }),
        _ => None,
    };
    Fields {
        version: match m {
            VersionedMessage::Legacy(_) => "legacy".into(),
            VersionedMessage::V0(_) => "0".into(),
            VersionedMessage::V1(_) => "1".into(),
        },
        num_signatures: tx.signatures.len(),
        header: [
            m.header().num_required_signatures,
            m.header().num_readonly_signed_accounts,
            m.header().num_readonly_unsigned_accounts,
        ],
        account_keys: m.static_account_keys().iter().map(|k| k.to_string()).collect(),
        recent_blockhash: m.recent_blockhash().to_string(),
        instructions: m
            .instructions()
            .iter()
            .map(|ix| IxOut {
                program_id_index: ix.program_id_index,
                accounts: ix.accounts.clone(),
                data_hex: hex::encode(&ix.data),
            })
            .collect(),
        num_address_table_lookups: m.address_table_lookups().map(|l| l.len()).unwrap_or(0),
        config,
    }
}
