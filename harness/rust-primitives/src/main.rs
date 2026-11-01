//! Harness: `solana-primitives` (hewigovens), an independent Rust implementation of the
//! Solana transaction wire format (legacy, v0, v1 / SIMD-0385).
use serde::Serialize;
use solana_primitives::{VersionedMessage, VersionedTransaction};
use std::io::{BufRead, Write};

#[derive(serde::Deserialize)]
struct Vec_ {
    id: String,
    tx_hex: String,
}
#[derive(Serialize)]
struct Ix {
    program_id_index: u8,
    accounts: Vec<u8>,
    data_hex: String,
}
#[derive(Serialize)]
struct Cfg {
    priority_fee_lamports: Option<String>,
    compute_unit_limit: Option<u32>,
    loaded_accounts_data_size_limit: Option<u32>,
    heap_size: Option<u32>,
}
#[derive(Serialize)]
struct Fields {
    version: String,
    num_signatures: usize,
    header: [u8; 3],
    account_keys: Vec<String>,
    recent_blockhash: String,
    instructions: Vec<Ix>,
    num_address_table_lookups: usize,
    config: Option<Cfg>,
}
#[derive(Serialize)]
struct Res {
    id: String,
    harness: String,
    decode: bool,
    error: Option<String>,
    sanitize: Option<bool>,
    fields: Option<Fields>,
    reserialized_hex: Option<String>,
}

fn ixs(v: &[solana_primitives::CompiledInstruction]) -> Vec<Ix> {
    v.iter()
        .map(|i| Ix { program_id_index: i.program_id_index, accounts: i.accounts.clone(), data_hex: hex::encode(&i.data) })
        .collect()
}

fn fields(tx: &VersionedTransaction) -> Fields {
    let n = tx.signatures.len();
    match &tx.message {
        VersionedMessage::Legacy(m) => Fields {
            version: "legacy".into(),
            num_signatures: n,
            header: [m.header.num_required_signatures, m.header.num_readonly_signed_accounts, m.header.num_readonly_unsigned_accounts],
            account_keys: m.account_keys.iter().map(|k| k.to_base58()).collect(),
            recent_blockhash: bs58::encode(m.recent_blockhash).into_string(),
            instructions: ixs(&m.instructions),
            num_address_table_lookups: 0,
            config: None,
        },
        VersionedMessage::V0(m) => Fields {
            version: "0".into(),
            num_signatures: n,
            header: [m.header.num_required_signatures, m.header.num_readonly_signed_accounts, m.header.num_readonly_unsigned_accounts],
            account_keys: m.account_keys.iter().map(|k| k.to_base58()).collect(),
            recent_blockhash: bs58::encode(m.recent_blockhash).into_string(),
            instructions: ixs(&m.instructions),
            num_address_table_lookups: m.address_table_lookups.len(),
            config: None,
        },
        VersionedMessage::V1(m) => Fields {
            version: "1".into(),
            num_signatures: n,
            header: [m.header.num_required_signatures, m.header.num_readonly_signed_accounts, m.header.num_readonly_unsigned_accounts],
            account_keys: m.account_keys.iter().map(|k| k.to_base58()).collect(),
            recent_blockhash: bs58::encode(m.lifetime_specifier).into_string(),
            instructions: ixs(&m.instructions),
            num_address_table_lookups: 0,
            config: Some(Cfg {
                priority_fee_lamports: m.config.priority_fee.map(|x| x.to_string()),
                compute_unit_limit: m.config.compute_unit_limit,
                loaded_accounts_data_size_limit: m.config.loaded_accounts_data_size_limit,
                heap_size: m.config.heap_size,
            }),
        },
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let inp = std::fs::File::open(&args[1]).expect("vectors");
    let mut out = std::fs::File::create(&args[2]).expect("out");
    let version = env!("CARGO_PKG_VERSION_PRIMITIVES");
    let harness = format!("solana-primitives@{version}");
    for line in std::io::BufReader::new(inp).lines() {
        let line = line.unwrap();
        if line.trim().is_empty() {
            continue;
        }
        let v: Vec_ = serde_json::from_str(&line).unwrap();
        let bytes = hex::decode(&v.tx_hex).unwrap();
        let mut r = Res { id: v.id, harness: harness.clone(), decode: false, error: None, sanitize: None, fields: None, reserialized_hex: None };
        // deserialize() already sanitizes and enforces size + exact consumption.
        match VersionedTransaction::deserialize(&bytes) {
            Ok(tx) => {
                r.decode = true;
                r.sanitize = Some(tx.sanitize().is_ok());
                r.fields = Some(fields(&tx));
                r.reserialized_hex = Some(match tx.serialize() { Ok(b) => hex::encode(b), Err(e) => format!("ENCODE_ERROR: {e:?}") });
            }
            Err(e) => r.error = Some(format!("{e:?}")),
        }
        writeln!(out, "{}", serde_json::to_string(&r).unwrap()).unwrap();
    }
}
