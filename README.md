# txconform

**Conformance vectors and differential harnesses for the Solana transaction wire format
(legacy, v0 and the new v1 / SIMD-0385), judged by Agave's own parser.**

Transaction v1 went live on mainnet on 15 September 2026 (epoch 1035). It is a new
binary layout: `0x81` version byte, a config bitmask carrying compute-unit limit, loaded
accounts data size, heap size and the priority fee (now a total in lamports), fixed-size
instruction headers, signatures at the end, 4,096-byte limit, no address lookup tables.
Within weeks, more than twenty repositories shipped their own v1 decoder: `@solana/kit`
(PR #1951), `@solana/web3.js` 1.x and 3.x, `solders` (PR #182), `solana-go`, x402
(TypeScript, Go, Python), pay-kit (nine languages), Trezor firmware (PR #8055), Metaplex Umi,
`sava` (Java), Helius SDKs, Ledger `device-sdk-ts`, `solana-primitives`, connectorkit,
Chainlink CCIP tools, SolSharp, the Haskell SDK, and more (GitHub code search lists 247
repositories referencing SIMD-0385 on 1 Nov 2026). Others are still working on it: Ledger
`app-solana` #248 and Phantom `phantom-connect-sdk` #407 are open.

Validator clients (Agave, Firedancer, Mithril) have shared conformance fixtures
(`firedancer-io/solana-conformance`, Agave's `sol_compat` entrypoints). **Client SDKs and
wallets do not.** Each of them re-derived the format from the SIMD text, and no one
publishes a shared set of "this must decode / this must be rejected" wire vectors judged by
the validator. txconform is that set, plus the tooling to run it against any
implementation. See [Related work](#related-work) for what does exist and how it differs.

## What it does

```
oracle/        Rust. (1) spec-derived generator: encodes legacy/v0/v1 transactions from
               the specification text alone, with 41 one-rule-broken mutation classes;
               (2) judge: runs every vector through Agave's own code and records the verdict
               at each layer a validator applies.
harness/ts     @solana/kit, @solana/web3.js 3.x, @solana/web3.js 1.x
harness/py     solders
harness/go     solana-go v2
txconform.py   diff: SDK result vs Agave verdict -> results/REPORT.md
vectors/       vectors.jsonl (input) and oracle.jsonl (Agave verdicts + canonical fields)
```

The oracle is **not** the Rust SDK alone. It composes the three layers a transaction
passes before it can be in a block, and keeps each layer's answer:

| layer | crate | what it enforces |
|---|---|---|
| packet parse + sanitize | `agave-transaction-view` | structure, header consistency, index bounds, config mask (reserved bits, priority-fee bit pair), no trailing bytes, 1,232-byte limit for legacy/v0 |
| protocol limits | `solana-runtime-transaction` | 4,096 bytes, 12 signatures, 64 addresses, 64 instructions, heap 32–256 KiB in 1 KiB steps, duplicate addresses (`AccountLoadedTwice`) |
| SDK | `solana-transaction` | `VersionedTransaction` decode + `sanitize()`, re-serialization, message bytes and hash |

Crate versions in `oracle/Cargo.toml`: `agave-transaction-view` 9.0.0, `solana-runtime-transaction` 4.5.0-alpha.4 (the released line that matches the view crate; it is an alpha), `solana-transaction` / `solana-message` 6.0.0.

A vector is `valid` only if all three accept. The generator's own expectation (from the
SIMD text) agreed with Agave on all 461 vectors, so the vectors double as an executable
reading of SIMD-0385.

## Results (1 November 2026)

| harness | rejects valid | accepts invalid | field mismatch | round-trip not identical |
|---|---|---|---|---|
| `@solana/kit@8.6.0` | 0 | 23 | 0 | 0 |
| `@solana/web3.js@3.1.1` | 0 | 22 | 0 | 0 |
| `@solana/web3.js@1.99.0` | 99 | 21 | 0 | 113 |
| `solders@0.29.0` | 0 | 5 | 0 | 0 |
| `solana-go/v2@v2.2.0` | 0 | 3 | 0 | 0 |

461 vectors: 414 Agave-valid, 47 Agave-invalid, 56 classes. Full table per class in
[`results/REPORT.md`](results/REPORT.md); triage in [`FINDINGS.md`](FINDINGS.md).

"Accepts invalid" is not automatically a bug: codecs such as kit decode structure and leave
policy to the caller. It becomes a bug when the caller is a wallet, co-signer, paymaster or
payment facilitator that approves what it decoded. The report says which Agave layer
rejected each vector so a maintainer can decide.

## Run it

```
./run.sh                 # builds the oracle, regenerates vectors, runs all harnesses, writes results/REPORT.md
RANDOM_VECTORS=2000 ./run.sh
```

Add an implementation: write a program that reads `vectors/vectors.jsonl` and emits one
JSON line per vector with `{id, harness, decode, error, sanitize, fields, reserialized_hex}`
(see any harness for the `fields` shape), then pass its output to `txconform.py --results`.

## Vector classes

Valid: random v1 (any config subset, up to 4,096 bytes), random legacy, random v0 with
lookups, empty config, full config, u64::MAX / u32::MAX config values, heap 32 KiB and
256 KiB, priority-fee-only, 12 signatures + 64 addresses + 64 instructions, 255 accounts in
one instruction, exactly 4,096 bytes, over 1,232 bytes, zero instructions, ComputeBudget
instructions present in a v1 transaction (must be ignored for config), legacy at 1,232
bytes, v0 instruction using looked-up accounts.

Invalid (one rule each): reserved mask bit (with and without its value word), half of the
priority-fee bit pair, heap not a 1 KiB multiple / below 32 KiB / above 256 KiB / zero,
duplicate address, no writable signer, zero signatures, header exceeds addresses,
instruction account index out of bounds, program index out of bounds, program is the fee
payer, 13 signatures, 65 addresses, 65 instructions, 4,097 bytes, trailing byte,
truncation at 9 offsets, missing signature, extra signature, version byte 0x82 / 0xFF, v1
body with 0x80 prefix, v1 message inside a legacy envelope, instruction data_len overflow,
legacy over 1,232 bytes, legacy duplicate address, legacy program-is-fee-payer,
non-canonical compact-u16 signature counts (`81 00`, `80 01`), v0 lookup that loads
nothing, v0 program from a lookup, legacy envelope with a 0x81 message, empty input,
single bytes, zero addresses.

## Related work

| project | what it checks | judge | why it does not cover this |
|---|---|---|---|
| `firedancer-io/solana-conformance` (archived 16 Apr 2026), Agave `sol_compat` entrypoints | validator clients against each other (syscalls, instructions, txn execution) | Agave | validator-only; client SDKs and wallets are not testees |
| `solana-foundation/pay-kit` `harness/` (conformance vectors, PR #387 merged 21 Oct 2026) | pay-kit's own x402 and MPP payment protocol SDKs in 9 languages; the v1 cases cover version negotiation, admission order, per-version size limit and compute-budget rules | pay-kit's Rust implementation (built on the Rust SDK) | protocol-level and curated ("not a fuzzer"); no rulings on config-mask bits, trailing bytes, heap bounds, count limits or duplicates; does not test kit, web3.js, solders, solana-go or hardware wallets; a Rust-SDK judge inherits the Rust SDK's laxness (F6) |
| `hewigovens/solana-primitives` PR #25 (Sep 2026) | one Rust crate, differential against `solana-transaction` 5.0 (about 5 M decode cases) | Rust SDK | single implementation, private harness, judge is the SDK not the validator |
| each SDK's own unit tests (kit, web3.js, solders, solana-go, Trezor, Ledger, Keystone) | that SDK against itself or a handful of SDK-generated fixtures | that SDK | no cross-implementation vectors; the bugs in FINDINGS.md all pass their own suites |

txconform's vectors are plain JSON with hex bytes and a per-layer Agave verdict, so they can
be dropped into any of the above (a pay-kit `harness/vectors` adapter is planned).

## License

Apache-2.0.
