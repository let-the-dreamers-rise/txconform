# Prior-art check: does the Solana ecosystem already cover this?

Written 1 Nov 2026. Reason this file exists: the Solana Foundation rejected Ash's earlier
proposal with "the ecosystem already covers that". Every claim below was checked on the
date given; links are to the pages read.

## The claim to defend

**No one publishes shared, validator-judged acceptance vectors for the Solana transaction
wire format (legacy, v0, v1 / SIMD-0385), and no harness runs the client SDKs and wallets
against the validator's parser.** txconform does both, and found divergences in all five
SDKs it tested on day one.

## What exists (checked 1 Nov 2026)

| # | Thing | What it is | Covers wire-format acceptance by client SDKs? | Source |
|---|---|---|---|---|
| 1 | `firedancer-io/solana-conformance` | Validator-vs-validator conformance fuzzing (syscalls, instructions, transaction execution, ELF loading) via `sol_compat_*` entrypoints | **No.** Testees are validator clients (Agave, Firedancer). Archived 16 Apr 2026. | github.com/firedancer-io/solana-conformance (archive banner); Agave wiki "Conformance Testing" |
| 2 | Agave wiki: Conformance Testing | Lists the 10 `sol_compat` entrypoints Agave exports for #1 | **No**, same scope as #1 | github.com/anza-xyz/agave/wiki |
| 3 | `solana-foundation/pay-kit` `harness/` conformance vectors (PR #387, merged 21 Oct 2026 by Ludovic Galabru, Solana Foundation) | 619 vectors for pay-kit's x402 and MPP payment-protocol SDKs (TS, Rust, Go, Python, Ruby, PHP, Lua, Kotlin, Swift). Transaction-v1 cases: version negotiation (`transactionVersions` array), admission before instruction checks, per-version size limit, header compute-budget rules (rulings T1-T4). | **Partly, for pay-kit only.** Judge is pay-kit's Rust implementation, which sits on the Rust SDK. README: "Not a fuzzer, scenarios are curated". RULINGS.md has no ruling on config-mask bits, trailing bytes, heap bounds, signature/address/instruction count limits or duplicate addresses. Testees are pay-kit's SDKs, not `@solana/kit`, web3.js, solders, solana-go, Ledger, Trezor, Keystone or Brave. | github.com/solana-foundation/pay-kit/pull/387; harness/expectations/RULINGS.md; harness/README.md |
| 4 | `hewigovens/solana-primitives` PR #25 (merged 18 Sep 2026) | A Rust primitives crate adding v1; author reports differential fuzzing against `solana-transaction` 5.0 (about 5 M decode cases, 0 mismatches) | **No.** One crate, private harness, judge is the Rust SDK. FINDINGS.md F6 shows the Rust SDK's `sanitize()` accepts 13 signatures, 65 addresses, 4,097 bytes, trailing bytes and duplicates that the validator rejects, so "0 mismatches with the SDK" does not mean "matches the validator". | github.com/hewigovens/solana-primitives/pull/25 |
| 5 | `solana-foundation/connectorkit` PR #64 | Adds v1 to a wallet connector; 33 tests on "wire-correct fixtures compiled via kit" | **No.** Fixtures are kit-generated valid transactions; no reject vectors; judge is kit. | github.com/solana-foundation/connectorkit/pull/64 |
| 6 | Each SDK's own tests: `@solana/kit`, `@solana/web3.js`, `solders`, `solana-go`, Trezor firmware, Ledger `app-solana`, Keystone, Brave, BitGoJS, Umi, `web3-core` | Unit tests written by the same team that wrote the parser | **No.** Self-judged, no shared vectors. All five bugs in FINDINGS.md pass the affected SDK's own suite. | each repo |
| 7 | solana.com "Larger Transaction Sizes" upgrade guide | Prose checklist for wallets and dApps (read config from the mask, not from ComputeBudget instructions, etc.) | **No.** Guidance, no vectors, no harness. | solana.com/upgrades/larger-transaction-sizes |
| 8 | SIMD-0385 text and `solana-transaction` / `agave-transaction-view` source | The specification and the reference implementation | **No.** The spec's constraint table (64 addresses, 12 signatures, 64 instructions) and the wire format (96, 42, 255) differ; where the limits are enforced is only discoverable by reading Agave's runtime crate. txconform records that layer per vector. | SIMD-0385; Agave source |
| 9 | Solana Foundation RFP board (Airtable) | Open RFPs (indexer tooling, Codama renderers, math library, historical state verification…) | Not fetchable from this environment (robots). Searched the public summaries (Colosseum blog, mpost): no RFP for transaction-format conformance or SDK test vectors. | blog.colosseum.com Solana RFPs post |

## Evidence that the gap costs real projects

All checked 31 Oct to 1 Nov 2026.

- **Trezor** `trezor-firmware` PR #8055 (merged 21 Oct 2026): while adding v1, the team also had to fix compact-u16 strictness and an instruction-misalignment bug in the existing parser.
- **Ledger** `app-solana` issue #248 (opened 2 Sep 2026, still open 1 Nov): "A SIMD-0385 message is therefore treated as a malformed v0 tx"; the C parser is independent of every SDK and will be re-derived again.
- **Phantom** `phantom-connect-sdk` issue #407 (opened 2 Oct 2026, open): cannot sign v1 transactions.
- **Anchorage** `visualsign-parser` PR #558 (13 Oct 2026, open): "fix(solana): parse instruction counts of 128 and up", a compact-u16 bug in a production signing parser; PR #567 fixed a size / v1 gate bug. **Gem wallet** issue #1266: another independent re-derivation.
- **pay-kit** FINDINGS.md (27 Oct 2026): Go, Swift, PHP, Ruby and Lua pay-kit SDKs "decode only legacy and v0"; TypeScript and Python v1 codecs were added 27 Oct. Five more parsers being written in the same month, by the Foundation's own team, with no validator-judged reject vectors to run against.
- **CVE-2024-30253** (`@solana/web3.js`): a past wire-level parsing flaw in the most used SDK.
- **txconform day-one findings** (1 Nov 2026): kit 8.6 accepts reserved config-mask bits; kit, web3.js 3, solders and the Rust SDK accept trailing bytes and an extra signature; solana-go misses the 1,232 / 4,096 size limits and legacy duplicates; web3.js 1.99 throws on valid v1 with a priority fee above 2^53; solders documents the fee in micro-lamports. Five SDKs, five divergence classes, none caught by the SDKs' own tests.

## Why "run an AI yourself" does not replace it

The Foundation could ask an AI to write a v1 parser test. What it cannot get that way:
- An **Agave-judged** verdict per vector per layer (packet view, runtime static, runtime resolved, SDK), produced by compiling and running Agave's crates, not by reading them.
- A **maintained** set that reruns against each SDK release weekly, with a baseline gate that fails on new divergences (`.github/workflows/conformance.yml`, `tools/check_baseline.py`).
- **Upstream fixes** landed and vectors adopted into the SDKs' own CI, which is people-work with maintainers.
- Harnesses for **hardware-wallet firmware** (Ledger Speculos, Trezor emulator) and the **RPC path** against a local validator, which need real engineering time, not a prompt.

## Where the claim is weak, stated plainly

- pay-kit's harness is the closest thing and it is the Foundation's own. The proposal must present txconform as the wire-format layer *under* it, and offer the vectors in pay-kit's `harness/vectors` format as a milestone deliverable, not as a competitor.
- "20+ independent parsers" is a count of repositories observed with v1 code; it is not a count of distinct hand-written decoders (some wrap kit or the Rust SDK). The README says "more than twenty independent parsers shipped v1 support"; the proposal uses "more than twenty repositories".
- None of the five findings is a fund-loss bug today. The pitch rests on forward compatibility (reserved bits), relay/verification correctness (trailing bytes), and robustness (web3.js 1.x), and says so.
