//! Spec-derived wire encoders for Solana transactions (legacy, v0, v1).
//!
//! These encoders are written from the specifications only (legacy/v0 wire
//! format as documented in the Solana docs, SIMD-0385 for v1) and deliberately
//! do NOT use the Solana SDK. The oracle (judge.rs) then checks every vector
//! against Agave's own parser. Any disagreement between this generator's
//! `expect_valid` flag and Agave is itself a finding (see FINDINGS.md).

use rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};

pub const V1_PREFIX: u8 = 0x81;
pub const V0_PREFIX: u8 = 0x80;
pub const PACKET_DATA_SIZE: usize = 1232;
pub const V1_MAX_SIZE: usize = 4096;
pub const V1_MAX_SIGNATURES: usize = 12;
pub const V1_MAX_ADDRESSES: usize = 64;
pub const V1_MAX_INSTRUCTIONS: usize = 64;
pub const MIN_HEAP: u32 = 32 * 1024;
pub const MAX_HEAP: u32 = 256 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ix {
    pub program_id_index: u8,
    pub accounts: Vec<u8>,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub priority_fee: Option<u64>,
    pub compute_unit_limit: Option<u32>,
    pub loaded_accounts_data_size_limit: Option<u32>,
    pub heap_size: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lookup {
    pub key: [u8; 32],
    pub writable: Vec<u8>,
    pub readonly: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Tx {
    pub version: Version,
    pub num_required_signatures: u8,
    pub num_readonly_signed: u8,
    pub num_readonly_unsigned: u8,
    pub blockhash: [u8; 32],
    pub keys: Vec<[u8; 32]>,
    pub ixs: Vec<Ix>,
    pub lookups: Vec<Lookup>,
    pub config: Config,
    pub signatures: Vec<[u8; 64]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Version {
    Legacy,
    V0,
    V1,
}

/// compact-u16 as specified (canonical encoding).
pub fn compact_u16(v: u16, out: &mut Vec<u8>) {
    let mut rem = v;
    loop {
        let mut elem = (rem & 0x7f) as u8;
        rem >>= 7;
        if rem == 0 {
            out.push(elem);
            break;
        }
        elem |= 0x80;
        out.push(elem);
    }
}

pub fn config_mask(c: &Config) -> u32 {
    let mut m = 0u32;
    if c.priority_fee.is_some() {
        m |= 0b11;
    }
    if c.compute_unit_limit.is_some() {
        m |= 0b100;
    }
    if c.loaded_accounts_data_size_limit.is_some() {
        m |= 0b1000;
    }
    if c.heap_size.is_some() {
        m |= 0b10000;
    }
    m
}

impl Tx {
    /// Message bytes (what gets signed).
    pub fn message_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match self.version {
            Version::Legacy | Version::V0 => {
                if self.version == Version::V0 {
                    out.push(V0_PREFIX);
                }
                out.extend([
                    self.num_required_signatures,
                    self.num_readonly_signed,
                    self.num_readonly_unsigned,
                ]);
                compact_u16(self.keys.len() as u16, &mut out);
                for k in &self.keys {
                    out.extend(k);
                }
                out.extend(self.blockhash);
                compact_u16(self.ixs.len() as u16, &mut out);
                for ix in &self.ixs {
                    out.push(ix.program_id_index);
                    compact_u16(ix.accounts.len() as u16, &mut out);
                    out.extend(&ix.accounts);
                    compact_u16(ix.data.len() as u16, &mut out);
                    out.extend(&ix.data);
                }
                if self.version == Version::V0 {
                    compact_u16(self.lookups.len() as u16, &mut out);
                    for l in &self.lookups {
                        out.extend(l.key);
                        compact_u16(l.writable.len() as u16, &mut out);
                        out.extend(&l.writable);
                        compact_u16(l.readonly.len() as u16, &mut out);
                        out.extend(&l.readonly);
                    }
                }
            }
            Version::V1 => {
                // SIMD-0385 layout
                out.push(V1_PREFIX);
                out.extend([
                    self.num_required_signatures,
                    self.num_readonly_signed,
                    self.num_readonly_unsigned,
                ]);
                out.extend(config_mask(&self.config).to_le_bytes());
                out.extend(self.blockhash);
                out.push(self.ixs.len() as u8);
                out.push(self.keys.len() as u8);
                for k in &self.keys {
                    out.extend(k);
                }
                if let Some(v) = self.config.priority_fee {
                    out.extend(v.to_le_bytes());
                }
                if let Some(v) = self.config.compute_unit_limit {
                    out.extend(v.to_le_bytes());
                }
                if let Some(v) = self.config.loaded_accounts_data_size_limit {
                    out.extend(v.to_le_bytes());
                }
                if let Some(v) = self.config.heap_size {
                    out.extend(v.to_le_bytes());
                }
                for ix in &self.ixs {
                    out.push(ix.program_id_index);
                    out.push(ix.accounts.len() as u8);
                    out.extend((ix.data.len() as u16).to_le_bytes());
                }
                for ix in &self.ixs {
                    out.extend(&ix.accounts);
                    out.extend(&ix.data);
                }
            }
        }
        out
    }

    pub fn tx_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match self.version {
            Version::Legacy | Version::V0 => {
                compact_u16(self.signatures.len() as u16, &mut out);
                for s in &self.signatures {
                    out.extend(s);
                }
                out.extend(self.message_bytes());
            }
            Version::V1 => {
                out.extend(self.message_bytes());
                for s in &self.signatures {
                    out.extend(s);
                }
            }
        }
        out
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vector {
    pub id: String,
    pub class: String,
    /// What the spec says: should a validator accept this transaction's format?
    pub expect_valid: bool,
    pub spec: String,
    pub note: String,
    pub tx_hex: String,
    /// Loaded addresses for v0 lookups (needed to resolve), hex-encoded 32-byte keys.
    #[serde(default)]
    pub loaded_writable_hex: Vec<String>,
    #[serde(default)]
    pub loaded_readonly_hex: Vec<String>,
}

pub struct Gen {
    pub rng: ChaCha20Rng,
    pub out: Vec<Vector>,
    counter: u64,
}

impl Gen {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha20Rng::seed_from_u64(seed),
            out: Vec::new(),
            counter: 0,
        }
    }

    fn key(&mut self) -> [u8; 32] {
        let mut k = [0u8; 32];
        self.rng.fill_bytes(&mut k);
        k
    }
    fn sig(&mut self) -> [u8; 64] {
        let mut s = [0u8; 64];
        self.rng.fill_bytes(&mut s);
        s
    }

    fn push(&mut self, class: &str, expect_valid: bool, spec: &str, note: &str, bytes: Vec<u8>) {
        self.push_full(class, expect_valid, spec, note, bytes, vec![], vec![]);
    }

    fn push_full(
        &mut self,
        class: &str,
        expect_valid: bool,
        spec: &str,
        note: &str,
        bytes: Vec<u8>,
        lw: Vec<[u8; 32]>,
        lr: Vec<[u8; 32]>,
    ) {
        self.counter += 1;
        self.out.push(Vector {
            id: format!("{}-{:05}", class, self.counter),
            class: class.to_string(),
            expect_valid,
            spec: spec.to_string(),
            note: note.to_string(),
            tx_hex: hex::encode(bytes),
            loaded_writable_hex: lw.iter().map(hex::encode).collect(),
            loaded_readonly_hex: lr.iter().map(hex::encode).collect(),
        });
    }

    /// A random, well-formed v1 transaction that fits in `max_size` bytes.
    pub fn random_v1(&mut self, max_size: usize) -> Tx {
        loop {
            let num_keys = self.rng.gen_range(2..=V1_MAX_ADDRESSES) as u8;
            let num_sigs = self.rng.gen_range(1..=(num_keys.min(V1_MAX_SIGNATURES as u8))) as u8;
            let ro_signed = self.rng.gen_range(0..num_sigs);
            let ro_unsigned = self.rng.gen_range(0..=(num_keys - num_sigs));
            // program ids must be non-signer? No: spec only says program_id_index != 0 and < num_keys.
            let num_ixs = self.rng.gen_range(0..=8usize);
            let mut ixs = Vec::new();
            for _ in 0..num_ixs {
                let n_acc = self.rng.gen_range(0..=6usize);
                let accounts = (0..n_acc).map(|_| self.rng.gen_range(0..num_keys)).collect();
                let dlen = self.rng.gen_range(0..=64usize);
                let mut data = vec![0u8; dlen];
                self.rng.fill_bytes(&mut data);
                ixs.push(Ix {
                    program_id_index: self.rng.gen_range(1..num_keys),
                    accounts,
                    data,
                });
            }
            let config = Config {
                priority_fee: if self.rng.gen_bool(0.5) { Some(self.rng.gen()) } else { None },
                compute_unit_limit: if self.rng.gen_bool(0.7) { Some(self.rng.gen()) } else { None },
                loaded_accounts_data_size_limit: if self.rng.gen_bool(0.5) { Some(self.rng.gen()) } else { None },
                heap_size: if self.rng.gen_bool(0.4) {
                    Some(self.rng.gen_range(32..=256) * 1024)
                } else {
                    None
                },
            };
            let keys = (0..num_keys).map(|_| self.key()).collect();
            let signatures = (0..num_sigs).map(|_| self.sig()).collect();
            let tx = Tx {
                version: Version::V1,
                num_required_signatures: num_sigs,
                num_readonly_signed: ro_signed,
                num_readonly_unsigned: ro_unsigned,
                blockhash: self.key(),
                keys,
                ixs,
                lookups: vec![],
                config,
                signatures,
            };
            if tx.tx_bytes().len() <= max_size {
                return tx;
            }
        }
    }

    pub fn random_legacy(&mut self, v0: bool) -> Tx {
        loop {
            let num_keys = self.rng.gen_range(2..=20) as u8;
            let num_sigs = self.rng.gen_range(1..=num_keys.min(8)) as u8;
            let ro_signed = self.rng.gen_range(0..num_sigs);
            let ro_unsigned = self.rng.gen_range(0..=(num_keys - num_sigs));
            let lookups: Vec<Lookup> = if v0 {
                (0..self.rng.gen_range(0..=2usize))
                    .map(|_| {
                        let nw = self.rng.gen_range(0..=3usize);
                        let nr = self.rng.gen_range(if nw == 0 { 1 } else { 0 }..=3usize);
                        Lookup {
                            key: self.key(),
                            writable: (0..nw).map(|_| self.rng.gen_range(0..200)).collect(),
                            readonly: (0..nr).map(|_| self.rng.gen_range(0..200)).collect(),
                        }
                    })
                    .collect()
            } else {
                vec![]
            };
            let total_keys = num_keys as usize
                + lookups.iter().map(|l| l.writable.len() + l.readonly.len()).sum::<usize>();
            let num_ixs = self.rng.gen_range(0..=6usize);
            let mut ixs = Vec::new();
            for _ in 0..num_ixs {
                let n_acc = self.rng.gen_range(0..=6usize);
                let accounts = (0..n_acc).map(|_| self.rng.gen_range(0..total_keys) as u8).collect();
                let dlen = self.rng.gen_range(0..=48usize);
                let mut data = vec![0u8; dlen];
                self.rng.fill_bytes(&mut data);
                ixs.push(Ix {
                    program_id_index: self.rng.gen_range(1..num_keys),
                    accounts,
                    data,
                });
            }
            let keys = (0..num_keys).map(|_| self.key()).collect();
            let signatures = (0..num_sigs).map(|_| self.sig()).collect();
            let tx = Tx {
                version: if v0 { Version::V0 } else { Version::Legacy },
                num_required_signatures: num_sigs,
                num_readonly_signed: ro_signed,
                num_readonly_unsigned: ro_unsigned,
                blockhash: self.key(),
                keys,
                ixs,
                lookups,
                config: Config::default(),
                signatures,
            };
            if tx.tx_bytes().len() <= PACKET_DATA_SIZE {
                return tx;
            }
        }
    }

    fn loaded_for(&mut self, tx: &Tx) -> (Vec<[u8; 32]>, Vec<[u8; 32]>) {
        let nw: usize = tx.lookups.iter().map(|l| l.writable.len()).sum();
        let nr: usize = tx.lookups.iter().map(|l| l.readonly.len()).sum();
        ((0..nw).map(|_| self.key()).collect(), (0..nr).map(|_| self.key()).collect())
    }

    pub fn generate(&mut self, n_random: usize) {
        let s385 = "SIMD-0385";
        // ---- valid random corpora
        for _ in 0..n_random {
            let tx = self.random_v1(V1_MAX_SIZE);
            self.push("v1-random", true, s385, "random well-formed v1 transaction", tx.tx_bytes());
        }
        for _ in 0..n_random / 2 {
            let tx = self.random_legacy(false);
            self.push("legacy-random", true, "legacy wire format", "random well-formed legacy transaction", tx.tx_bytes());
        }
        for _ in 0..n_random / 2 {
            let tx = self.random_legacy(true);
            let (lw, lr) = self.loaded_for(&tx);
            self.push_full("v0-random", true, "v0 wire format (versioned transactions)", "random well-formed v0 transaction with lookups", tx.tx_bytes(), lw, lr);
        }

        // ---- v1 boundary: valid
        let mut base = self.random_v1(1000);
        base.config = Config::default();
        base.ixs.truncate(2);
        let b = base.clone();
        self.push("v1-empty-config", true, s385, "mask = 0, no config values; defaults apply (CU limit 0!)", b.tx_bytes());

        let mut t = base.clone();
        t.config = Config { priority_fee: Some(1), compute_unit_limit: Some(1_400_000), loaded_accounts_data_size_limit: Some(32 * 1024), heap_size: Some(MIN_HEAP) };
        self.push("v1-full-config", true, s385, "all four config fields present (mask 0b11111)", t.tx_bytes());

        let mut t = base.clone();
        t.config = Config { priority_fee: Some(u64::MAX), compute_unit_limit: Some(u32::MAX), loaded_accounts_data_size_limit: Some(u32::MAX), heap_size: Some(MAX_HEAP) };
        self.push("v1-config-max-values", true, s385, "u64::MAX priority fee lamports, u32::MAX limits, 256 KiB heap: format-valid (SDKs must not clamp/overflow)", t.tx_bytes());

        let mut t = base.clone();
        t.config = Config { heap_size: Some(MIN_HEAP), ..Default::default() };
        self.push("v1-heap-min", true, s385, "heap exactly 32 KiB", t.tx_bytes());
        let mut t = base.clone();
        t.config = Config { heap_size: Some(MAX_HEAP), ..Default::default() };
        self.push("v1-heap-max", true, s385, "heap exactly 256 KiB", t.tx_bytes());

        // priority fee only: tests that SDK reads the 8-byte field as two config words
        let mut t = base.clone();
        t.config = Config { priority_fee: Some(0x0102030405060708), ..Default::default() };
        self.push("v1-priority-fee-only", true, s385, "only bits 0,1 set; value is 8-byte LE u64 total lamports", t.tx_bytes());

        // limits
        let mut t = self.random_v1(2000);
        t.keys = (0..64).map(|_| self.key()).collect();
        t.num_required_signatures = 12;
        t.num_readonly_signed = 0;
        t.num_readonly_unsigned = 20;
        t.signatures = (0..12).map(|_| self.sig()).collect();
        t.ixs = (0..64).map(|i| Ix { program_id_index: 13 + (i % 50) as u8, accounts: vec![0, 1], data: vec![i as u8] }).collect();
        t.config = Config { compute_unit_limit: Some(1), ..Default::default() };
        self.push("v1-at-limits", true, s385, "12 signatures, 64 addresses, 64 instructions: all at the maximum", t.tx_bytes());

        // 255 accounts per instruction (indexes may repeat)
        let mut t = base.clone();
        t.ixs = vec![Ix { program_id_index: 1, accounts: (0..255).map(|i| (i % t.keys.len()) as u8).collect(), data: vec![] }];
        self.push("v1-ix-255-accounts", true, s385, "one instruction with 255 account indexes", t.tx_bytes());

        // exactly 4096 bytes
        let mut t = base.clone();
        t.ixs = vec![Ix { program_id_index: 1, accounts: vec![0], data: vec![] }];
        let cur = t.tx_bytes().len();
        t.ixs[0].data = vec![0xAB; V1_MAX_SIZE - cur];
        assert_eq!(t.tx_bytes().len(), V1_MAX_SIZE);
        self.push("v1-size-4096", true, s385, "exactly 4096 bytes (max)", t.tx_bytes());

        // over the legacy packet size but under 4096: the whole point of v1
        let mut t = base.clone();
        t.ixs = vec![Ix { program_id_index: 1, accounts: vec![0], data: vec![0xCD; 1500] }];
        self.push("v1-size-over-1232", true, s385, "v1 transaction larger than legacy PACKET_DATA_SIZE", t.tx_bytes());

        // zero instructions
        let mut t = base.clone();
        t.ixs = vec![];
        self.push("v1-zero-instructions", true, s385, "num_instructions = 0", t.tx_bytes());

        // ComputeBudget instruction present in a v1 tx: valid, must be ignored for config
        let mut t = base.clone();
        let cb_program: [u8; 32] = bs58::decode("ComputeBudget111111111111111111111111111111").into_vec().unwrap().try_into().unwrap();
        t.keys.push(cb_program);
        t.num_readonly_unsigned = t.num_readonly_unsigned.saturating_add(1);
        let cb_idx = (t.keys.len() - 1) as u8;
        // SetComputeUnitLimit(2) = 0x02 + u32 LE ; SetComputeUnitPrice(3) = 0x03 + u64 LE
        t.ixs = vec![
            Ix { program_id_index: cb_idx, accounts: vec![], data: { let mut d = vec![2u8]; d.extend(999_999u32.to_le_bytes()); d } },
            Ix { program_id_index: cb_idx, accounts: vec![], data: { let mut d = vec![3u8]; d.extend(777u64.to_le_bytes()); d } },
        ];
        t.config = Config { compute_unit_limit: Some(50_000), priority_fee: Some(5), ..Default::default() };
        self.push("v1-computebudget-ix-ignored", true, s385, "ComputeBudget instructions present: valid no-ops; effective CU limit is 50000 and fee is 5 lamports from the config, NOT 999999/777 from the instructions", t.tx_bytes());

        // ---- v1 invalid: one rule broken each
        let bad = |g: &mut Gen, class: &str, note: &str, bytes: Vec<u8>| g.push(class, false, s385, note, bytes);

        let mut t = base.clone();
        t.config = Config { compute_unit_limit: Some(1), ..Default::default() };
        let mut b = t.tx_bytes();
        // set an unknown mask bit (bit 5)
        b[4] |= 0b10_0000;
        bad(self, "v1-mask-unknown-bit", "config mask bit 5 set (reserved)", b);

        // same, but with the 4-byte value word an unknown bit would carry (spec: popcount values)
        let mut b = t.tx_bytes();
        b[4] |= 0b10_0000;
        let keys_end = 1 + 3 + 4 + 32 + 1 + 1 + 32 * t.keys.len();
        b.splice(keys_end + 4..keys_end + 4, 0xDEADBEEFu32.to_le_bytes());
        bad(self, "v1-mask-unknown-bit-with-value", "reserved mask bit 5 set AND its 4-byte value word present (future-SIMD shape); decoders that ignore unknown bits misalign the instruction headers", b);

        let mut b = t.tx_bytes();
        b[4] |= 0b1; // only bit 0 of the priority-fee pair
        // append 4 bytes of value so lengths line up with popcount
        let keys_end = 1 + 3 + 4 + 32 + 1 + 1 + 32 * t.keys.len();
        b.splice(keys_end..keys_end, [0u8; 4]);
        bad(self, "v1-mask-half-priority-fee", "only bit 0 of the two priority-fee bits set", b);

        let mut t = base.clone();
        t.config = Config { heap_size: Some(MIN_HEAP + 1), ..Default::default() };
        bad(self, "v1-heap-not-1k-multiple", "heap 32769 not a multiple of 1024", t.tx_bytes());
        let mut t = base.clone();
        t.config = Config { heap_size: Some(MIN_HEAP - 1024), ..Default::default() };
        bad(self, "v1-heap-below-min", "heap 31 KiB < 32 KiB", t.tx_bytes());
        let mut t = base.clone();
        t.config = Config { heap_size: Some(MAX_HEAP + 1024), ..Default::default() };
        bad(self, "v1-heap-above-max", "heap 257 KiB > 256 KiB", t.tx_bytes());
        let mut t = base.clone();
        t.config = Config { heap_size: Some(0), ..Default::default() };
        bad(self, "v1-heap-zero", "heap bit set with value 0", t.tx_bytes());

        let mut t = base.clone();
        t.keys[1] = t.keys[0];
        bad(self, "v1-duplicate-address", "address 1 duplicates address 0", t.tx_bytes());

        let mut t = base.clone();
        t.num_readonly_signed = t.num_required_signatures;
        bad(self, "v1-no-writable-signer", "num_readonly_signed == num_required_signatures (fee payer would be read-only)", t.tx_bytes());

        let mut t = base.clone();
        t.num_required_signatures = 0;
        t.num_readonly_signed = 0;
        t.signatures.clear();
        bad(self, "v1-zero-signatures", "num_required_signatures = 0", t.tx_bytes());

        let mut t = base.clone();
        t.num_readonly_unsigned = (t.keys.len() as u8) - t.num_required_signatures + 1;
        bad(self, "v1-header-exceeds-addresses", "required_signatures + readonly_unsigned > num_addresses", t.tx_bytes());

        let mut t = base.clone();
        t.ixs = vec![Ix { program_id_index: 1, accounts: vec![t.keys.len() as u8], data: vec![] }];
        bad(self, "v1-ix-account-index-oob", "instruction account index == num_addresses", t.tx_bytes());

        let mut t = base.clone();
        t.ixs = vec![Ix { program_id_index: t.keys.len() as u8, accounts: vec![0], data: vec![] }];
        bad(self, "v1-program-index-oob", "program_id_index == num_addresses", t.tx_bytes());

        let mut t = base.clone();
        t.ixs = vec![Ix { program_id_index: 0, accounts: vec![1], data: vec![] }];
        bad(self, "v1-program-is-fee-payer", "program_id_index = 0 (fee payer cannot be a program)", t.tx_bytes());

        let mut t = self.random_v1(1500);
        t.keys = (0..14).map(|_| self.key()).collect();
        t.num_required_signatures = 13;
        t.num_readonly_signed = 0;
        t.num_readonly_unsigned = 1;
        t.ixs = vec![Ix { program_id_index: 13, accounts: vec![0], data: vec![] }];
        t.signatures = (0..13).map(|_| self.sig()).collect();
        bad(self, "v1-13-signatures", "13 signatures > max 12", t.tx_bytes());

        let mut t = base.clone();
        t.keys = (0..65).map(|_| self.key()).collect();
        t.num_readonly_unsigned = 10;
        t.ixs = vec![Ix { program_id_index: 1, accounts: vec![64], data: vec![] }];
        bad(self, "v1-65-addresses", "65 addresses > max 64", t.tx_bytes());

        let mut t = base.clone();
        t.ixs = (0..65).map(|_| Ix { program_id_index: 1, accounts: vec![], data: vec![] }).collect();
        bad(self, "v1-65-instructions", "65 instructions > max 64", t.tx_bytes());

        let mut t = base.clone();
        t.ixs = vec![Ix { program_id_index: 1, accounts: vec![0], data: vec![] }];
        let cur = t.tx_bytes().len();
        t.ixs[0].data = vec![0xAB; V1_MAX_SIZE + 1 - cur];
        bad(self, "v1-size-4097", "4097 bytes > max 4096", t.tx_bytes());

        let mut b = base.tx_bytes();
        b.push(0x00);
        bad(self, "v1-trailing-byte", "one extra byte after the last signature", b);

        let b = base.tx_bytes();
        let n = b.len();
        for cut in [1usize, 4, 8, 40, 41, 42, 42 + 32, n - 64 - 1, n - 1] {
            if cut < n {
                bad(self, "v1-truncated", &format!("truncated to {cut} of {n} bytes"), b[..cut].to_vec());
            }
        }

        let mut t = base.clone();
        t.signatures.pop();
        bad(self, "v1-missing-signature", "fewer signatures than num_required_signatures", t.tx_bytes());
        let mut t = base.clone();
        let s = self.sig();
        t.signatures.push(s);
        bad(self, "v1-extra-signature", "one more signature than num_required_signatures (trailing 64 bytes)", t.tx_bytes());

        let mut b = base.tx_bytes();
        b[0] = 0x82;
        bad(self, "v1-version-byte-0x82", "version byte 0x82 (v2 does not exist)", b);
        let mut b = base.tx_bytes();
        b[0] = 0x80;
        bad(self, "v1-body-with-v0-prefix", "v1 body with 0x80 prefix: parses as a v0 message with absurd fields", b);
        let mut b = base.tx_bytes();
        b[0] = 0xFF;
        bad(self, "v1-version-byte-0xff", "version byte 0xFF", b);

        // v1 message in legacy envelope: [sig_count][sigs][0x81 ...]
        let mut b = Vec::new();
        compact_u16(base.signatures.len() as u16, &mut b);
        for s in &base.signatures {
            b.extend(s);
        }
        b.extend(base.message_bytes());
        bad(self, "v1-message-in-legacy-envelope", "signatures first, then a 0x81 message: not a valid v1 or legacy/v0 transaction", b);

        // instruction header says more data than present (lengths inconsistent)
        let mut t = base.clone();
        t.ixs = vec![Ix { program_id_index: 1, accounts: vec![0], data: vec![1, 2, 3] }];
        let mut b = t.tx_bytes();
        // instruction header data_len is 2 bytes LE after program_idx and num_accounts
        let hdr = 1 + 3 + 4 + 32 + 1 + 1 + 32 * t.keys.len() + 2;
        b[hdr] = 0xFF;
        b[hdr + 1] = 0xFF;
        bad(self, "v1-ix-data-len-overflow", "instruction header data_len 65535 but payload is 3 bytes (and signatures would be consumed as data)", b);

        // ---- legacy/v0 boundary vectors
        let l = self.random_legacy(false);
        let mut l2 = l.clone();
        l2.ixs = vec![Ix { program_id_index: 1, accounts: vec![0], data: vec![] }];
        let cur = l2.tx_bytes().len();
        l2.ixs[0].data = vec![0xEE; PACKET_DATA_SIZE - cur];
        // data length 2-byte compact u16 may shift by one; fix up
        while l2.tx_bytes().len() > PACKET_DATA_SIZE { l2.ixs[0].data.pop(); }
        self.push("legacy-size-max", true, "legacy: PACKET_DATA_SIZE 1232", &format!("legacy transaction of {} bytes", l2.tx_bytes().len()), l2.tx_bytes());
        let mut l3 = l2.clone();
        l3.ixs[0].data.extend([0xEE, 0xEE, 0xEE]);
        self.push("legacy-size-over-1232", false, "legacy: PACKET_DATA_SIZE 1232", &format!("legacy transaction of {} bytes (> 1232)", l3.tx_bytes().len()), l3.tx_bytes());

        let mut l4 = l.clone();
        l4.keys[1] = l4.keys[0];
        self.push("legacy-duplicate-address", false, "runtime: AccountLoadedTwice", "duplicate static address in a legacy tx (static sanitize passes; runtime rejects)", l4.tx_bytes());

        let mut l5 = l.clone();
        l5.ixs = vec![Ix { program_id_index: 0, accounts: vec![], data: vec![] }];
        self.push("legacy-program-is-fee-payer", false, "legacy sanitize", "program_id_index = 0", l5.tx_bytes());

        // non-canonical compact-u16 signature count: 0x81 0x00 would mean 1 in a lenient decoder,
        // but 0x81 is now the v1 discriminator.
        let mut b = Vec::new();
        b.push(0x81);
        b.push(0x00);
        for s in &l.signatures[..1] {
            b.extend(s);
        }
        let mut l6 = l.clone();
        l6.num_required_signatures = 1;
        l6.num_readonly_signed = 0;
        l6.signatures.truncate(1);
        b.extend(l6.message_bytes());
        self.push("legacy-noncanonical-sigcount-0x81", false, "compact-u16 canonical form; SIMD-0385 discriminator", "signature count encoded non-canonically as 81 00: must not be read as legacy with 1 signature", b);

        let mut b = Vec::new();
        b.push(0x80);
        b.push(0x01); // non-canonical encoding of 128
        self.push("legacy-noncanonical-sigcount-0x80-0x01", false, "compact-u16 canonical form", "two-byte compact-u16 for signature count (128) with no body", b);

        // v0 with a lookup that loads no accounts
        let mut v = self.random_legacy(true);
        v.lookups = vec![Lookup { key: self.key(), writable: vec![], readonly: vec![] }];
        let (lw, lr) = self.loaded_for(&v);
        self.push_full("v0-lookup-loads-nothing", false, "v0 sanitize", "address table lookup with zero writable and zero readonly indexes", v.tx_bytes(), lw, lr);

        // v0 where instruction program id points into lookup space
        let mut v = self.random_legacy(true);
        v.lookups = vec![Lookup { key: self.key(), writable: vec![1], readonly: vec![2] }];
        v.ixs = vec![Ix { program_id_index: v.keys.len() as u8, accounts: vec![0], data: vec![] }];
        let (lw, lr) = self.loaded_for(&v);
        self.push_full("v0-program-from-lookup", false, "v0 sanitize: program ids must be static", "program_id_index points at a looked-up address", v.tx_bytes(), lw, lr);

        // v0 instruction account index into loaded addresses: valid
        let mut v = self.random_legacy(true);
        v.lookups = vec![Lookup { key: self.key(), writable: vec![1, 2], readonly: vec![3] }];
        let nk = v.keys.len() as u8;
        v.ixs = vec![Ix { program_id_index: 1, accounts: vec![nk, nk + 1, nk + 2], data: vec![7] }];
        let (lw, lr) = self.loaded_for(&v);
        self.push_full("v0-ix-uses-loaded-accounts", true, "v0 wire format", "instruction accounts reference all three looked-up addresses", v.tx_bytes(), lw, lr);

        // v0 with a v1-style config? Not possible. v0 body with 0x81 prefix:
        let mut b = v.tx_bytes();
        let sig_end = 1 + 64 * v.signatures.len();
        b[sig_end] = 0x81;
        self.push("v0-message-prefix-0x81", false, "SIMD-0385", "legacy envelope whose message starts with 0x81", b);

        // empty / tiny inputs
        self.push("empty", false, "wire format", "zero bytes", vec![]);
        self.push("single-zero-byte", false, "wire format", "one byte: zero signatures, no message", vec![0]);
        self.push("single-0x81", false, "SIMD-0385", "one byte: v1 prefix only", vec![0x81]);
        let mut b = vec![0x81];
        b.extend([1, 0, 0]);
        b.extend(0u32.to_le_bytes());
        b.extend([0u8; 32]);
        b.extend([0, 0]);
        self.push("v1-header-zero-addresses", false, "SIMD-0385", "fixed header with num_addresses = 0 and 1 required signature", b);
    }
}

impl Gen {
    /// Mutation corpus: single-bit flips and truncations of well-formed transactions.
    /// The spec does not say what these should do, so `expect_valid` is a placeholder and
    /// `spec` starts with "mutation"; the judge skips the spec-vs-Agave check for them and
    /// Agave's verdict is the only ground truth. The point is coverage of every byte
    /// position with the validator's answer attached.
    pub fn mutate(&mut self, n_flips: usize) {
        let mut v1 = self.random_v1(600);
        v1.ixs.truncate(3);
        let mut v1c = v1.clone();
        v1c.config = Config { priority_fee: Some(5000), compute_unit_limit: Some(200_000), loaded_accounts_data_size_limit: Some(64 * 1024), heap_size: Some(MIN_HEAP) };
        let legacy = self.random_legacy(false);
        let v0 = self.random_legacy(true);
        let (v0w, v0r) = self.loaded_for(&v0);
        let templates: Vec<(&str, Vec<u8>, Vec<[u8; 32]>, Vec<[u8; 32]>)> = vec![
            ("v1", v1.tx_bytes(), vec![], vec![]),
            ("v1-config", v1c.tx_bytes(), vec![], vec![]),
            ("legacy", legacy.tx_bytes(), vec![], vec![]),
            ("v0", v0.tx_bytes(), v0w, v0r),
        ];
        // every truncation point of the small v1 and the legacy template
        for (name, bytes, lw, lr) in templates.iter() {
            if *name == "v1-config" || *name == "v0" { continue; }
            for cut in 0..bytes.len() {
                self.push_full(
                    &format!("mut-truncate-{name}"), false, "mutation: truncation",
                    &format!("{name} template cut to {cut} of {} bytes", bytes.len()),
                    bytes[..cut].to_vec(), lw.clone(), lr.clone(),
                );
            }
        }
        // every single-bit flip in the first 48 bytes of the full-config v1 template
        // (version, header, mask, first config words, lifetime specifier start)
        let v1bytes = templates[1].1.clone();
        for byte in 0..48.min(v1bytes.len()) {
            for bit in 0..8 {
                let mut b = v1bytes.clone();
                b[byte] ^= 1 << bit;
                self.push(
                    "mut-bitflip-v1-header", false, "mutation: single-bit flip",
                    &format!("v1-config template, byte {byte} bit {bit} flipped"), b,
                );
            }
        }
        for _ in 0..n_flips {
            let t = self.rng.gen_range(0..templates.len());
            let (name, bytes, lw, lr) = &templates[t];
            let byte = self.rng.gen_range(0..bytes.len());
            let bit = self.rng.gen_range(0..8);
            let mut b = bytes.clone();
            b[byte] ^= 1 << bit;
            self.push_full(
                &format!("mut-bitflip-{name}"), false, "mutation: single-bit flip",
                &format!("{name} template, byte {byte} bit {bit} flipped"), b, lw.clone(), lr.clone(),
            );
        }
    }
}

pub fn rng_u8(rng: &mut impl Rng, lo: u8, hi: u8) -> u8 {
    rng.gen_range(lo..=hi)
}
