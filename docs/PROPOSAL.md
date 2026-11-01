# txconform: the validator's own answer for every Solana transaction parser

Solana Foundation standard grant proposal, v2, 1 Nov 2026. Applicant: Ashwin Goyal (GitHub
`let-the-dreamers-rise`, letthedreamersrise@gmail.com, Navi Mumbai, India).
Repository: https://github.com/let-the-dreamers-rise/txconform (Apache-2.0, CI green).

## Why this is not something you already have, and not something an AI prompt gives you

The ecosystem already does conformance for validators (Firedancer and Mithril are checked
against Agave), and last month your own team merged a conformance harness into pay-kit
(PR #387, 619 vectors, nine SDKs). txconform is the layer none of those cover: **the transaction wire
format as read by client SDKs, wallets and payment verifiers, judged by Agave's own parser
crates.** pay-kit's vectors are payment-protocol cases judged by pay-kit's Rust
implementation, curated by hand, with no rulings on config-mask bits, trailing bytes, heap
bounds, count limits or duplicate addresses; its testees are pay-kit's SDKs, not kit, web3.js,
solders, the two Go SDKs or any wallet firmware. The validator-conformance work tests
validators. Each SDK tests itself. The full table is in `docs/PRIOR-ART.md`.

What an AI prompt cannot produce: a verdict per vector per validator layer from compiling and
running Agave's crates; a mutation corpus re-judged on every run; eight SDK harnesses that
already disagree with the validator in eight documented ways; a weekly CI run with a baseline
gate that fails on a regression in a new SDK release; vectors exported in pay-kit's format;
and the people-work of landing fixes upstream and getting the vectors into the SDKs' own CI.
All of that except the last exists today and is reproducible with one command.

## One paragraph

Transaction v1 (SIMD-0385) went live on mainnet on 15 September 2026 as a new binary layout.
GitHub code search lists 247 repositories referencing the SIMD; more than twenty ship their
own decoder (kit, web3.js 1.x and 3.x, solders, both Go SDKs, x402 and pay-kit SDKs, Trezor
firmware, Umi, sava, Helius SDKs, Ledger's device SDK); Ledger `app-solana` #248 and Phantom
`phantom-connect-sdk` #407 are still writing theirs. There is no shared set of "these bytes
must decode to exactly this / these must be rejected" vectors with the validator's answer
attached. txconform is that set and the tooling to run it: a spec-derived generator (valid
transactions plus 41 one-rule-broken classes plus thousands of bit-flip and truncation
mutations), Agave's crates as the judge with the verdict recorded at each layer, a 100-line
harness per SDK, a diff tool, a baseline gate, and CI. On 1 Nov 2026 it ran 3,012 vectors
through eight implementations and found a divergence in every one of them, and its RPC-path job
showed that Agave's own RPC accepts a transaction its block producer drops, returning a
signature for something that never lands.

## The problem, precisely

A Solana transaction is bytes a user signs. Three kinds of software read them and must agree:
the validator (decides inclusion), the SDKs that build, relay and verify them (kit, web3.js,
solders, solana-go, the Rust SDK, x402 facilitators, pay-kit servers, indexers), and the
signers that display them (Trezor, Keystone, Brave, Phantom, Ledger). If an SDK accepts what
the validator rejects, a payment is "verified" and never lands. If a signer reads a field
differently, the user signs something other than what was shown. If an SDK rejects what the
validator accepts, valid blocks break indexers and wallets. v1 added new ways to get each of
these wrong: a config bitmask with reserved bits, four optional words, a priority fee whose
unit changed (total lamports, not micro-lamports per CU), new limits (4,096 bytes, 12
signatures, 64 addresses, 64 instructions, heap 32 to 256 KiB in 1 KiB steps), ComputeBudget
instructions that became no-ops, and signatures moved to the end.

## What exists today (built 31 Oct to 1 Nov 2026, in the repository)

| Component | State |
|---|---|
| Generator (Rust, no SDK dependency) | 22 valid classes, 41 invalid classes derived from the SIMD text; mutation corpus: every truncation point of two templates, every header bit flip of a v1 template, 1,000 random bit flips |
| Oracle (Rust) | Runs each vector through `agave-transaction-view` (packet parse + sanitize), `solana-runtime-transaction` (static and resolved: limits, heap, duplicates) and the `solana-transaction` SDK; records each layer's verdict and canonical fields. Spec reading vs Agave: 0 disagreements on all hand-written vectors |
| Testees | `@solana/kit` 8.6.0, `@solana/web3.js` 3.1.1 and 1.99.0, `solders` 0.29.0, `solana-go` v2.2.0, `gagliardetto/solana-go` v1.25.0, Rust SDK `solana-transaction` 6.0.0, `solana-primitives` 0.3.0 |
| Diff + report | rejects-valid, accepts-invalid, field-mismatch, round-trip per testee per class; `results/REPORT.md` |
| CI | weekly against latest releases; baseline gate fails on any new divergence class; second job sends every vector to `simulateTransaction` on a `solana-test-validator` from the latest Agave release and runs an end-to-end send probe (does an RPC-accepted mutation land?) |
| pay-kit export | the Agave-rejected vectors in pay-kit's `verify-transaction` vector format (`expect reject / invalid-payload`), ready to propose upstream |
| Run time | about 15 s for the whole pipeline on a warm build |

Results on 1 Nov 2026 (3,012 vectors: 1,681 valid, 1,331 invalid):

| testee | rejects valid | accepts invalid | round-trip differs |
|---|---|---|---|
| Rust SDK `solana-transaction` 6.0.0 | 0 | 17 | 0 |
| `@solana/kit` 8.6.0 | 0 | 89 | 0 |
| `@solana/web3.js` 3.1.1 | 0 | 58 | 0 |
| `@solana/web3.js` 1.99.0 | 327 | 48 | 658 |
| `solders` 0.29.0 | 0 | 7 | 0 |
| `solana-go` v2.2.0 | 0 | 3 | 0 |
| `gagliardetto/solana-go` v1.25.0 | 0 | 3 | 0 |
| `solana-primitives` 0.3.0 | 0 | 1 | 0 |

Findings (`FINDINGS.md`, with the vector class that reproduces each and seven upstream issue
drafts in `upstream/`, including one for Anza):

| # | Finding |
|---|---|
| F1 | kit's compiled-message decoder accepts reserved config-mask bits; Agave rejects. A signer that shows the four known fields has signed over configuration it did not display. web3.js 3.x rejects one layer up |
| F2 | kit, web3.js 3.x, solders and the Rust SDK accept trailing bytes and an extra trailing signature; Agave requires the frame to end at the buffer. Anything verifying client bytes before relaying accepts bytes no validator will include |
| F3 | `solana-go` v2 `Sanitize()` passes 1,235-byte legacy, 4,097-byte v1 and legacy duplicate addresses |
| F4 | web3.js 1.x throws on valid v1 with a priority fee above 2^53 (327 vectors), cannot serialize v1, and names the field `priorityFee` where 3.x says `priorityFeeLamports` |
| F5 | solders documents the priority fee in micro-lamports; spec and every other SDK say total lamports |
| F6 | the Rust SDK's `sanitize()` is laxer than the validator (13 signatures, 65 addresses, 4,097 bytes, heap bounds, duplicates); ports that mirror the SDK inherit the gap. The oracle records which layer enforces each rule |
| F7 | `gagliardetto/solana-go` has exactly the three gaps of `solana-go` v2 |
| F8 | `solana-primitives` matches Agave on 3,011 of 3,012 vectors; the one gap is legacy duplicate addresses |
| F9 | **Agave's own RPC** (`sendTransaction`, preflight on) accepts a transaction with one trailing byte, returns a signature, and the block producer drops it: measured end to end on an Agave 4.4.0 test validator in CI. The client gets no error, ever. The RPC deserializes with the SDK path (F2/F6); the TPU uses the strict packet view |

None is a fund-loss bug today. F9 is a silent availability hole inside the node, and it shows
the thesis in one table: two layers of the same validator disagree about the same bytes. The
others are the divergences that become bugs when a reserved bit is defined, a facilitator
trusts its SDK, or a wallet shows a fee in the wrong unit. All eight
implementations pass their own test suites. The mutation corpus found no root cause beyond
these, which is evidence the hand-written classes cover the format; it stays in CI so new
releases are measured against 3,000 vectors.

## Why Solana, and why the Foundation

The format, the parsers and the judge are all Solana's; there is no chain-agnostic version of
this work. Direct beneficiaries are the Foundation's own kit, pay-kit and connectorkit; Anza,
whose crates are the de-facto spec; wallet vendors; payment integrators who verify
transactions before relaying them; indexers. No single SDK team can maintain a cross-SDK
suite, and a validator-judged vector set is a public good in the plain sense: JSON files
useful without the author.

## Deliverables and milestones

USD 30,000 over 16 weeks, paid per milestone on acceptance. Each milestone is files in the
public repository plus links to upstream PRs; nothing needs Foundation time beyond review.

### M1 (weeks 1-5, USD 8,000): vectors v1.0, eleven testees, fixes opened upstream

- Vector set v1.0 as versioned JSON (hex bytes, class, per-layer Agave verdict, canonical
  fields) with a written harness contract so any team adds a testee in about 100 lines.
- Generator: forward-compatibility set for each undefined config-mask bit; mutation corpus
  to at least 5,000 vectors per run; multi-byte and boundary mutations.
- New testees: `sava` (Java), Solana Mobile `web3-core` (Kotlin), Solana Swift, Solana Dart,
  Solnet (C#).
- Upstream: issues and, where accepted, fix PRs for F1-F8 (six drafts exist); pay-kit PR
  proposing the exported wire vectors.
- Acceptance: eleven harnesses green in CI; vectors v1.0 tagged; issue links in FINDINGS.md.

### M2 (weeks 6-11, USD 12,000): signers, payment stacks, RPC path

- Signers as testees: Ledger `app-solana` via Speculos (once #248 lands), Trezor firmware via
  the Trezor emulator, Keystone firmware parser, Brave wallet's C++ parser. For each:
  accept/reject per vector and the fields shown to the user, diffed against Agave's canonical
  fields. Any display mismatch goes to the vendor privately first.
- Payment stacks in verify mode: x402's SVM implementations (TypeScript, Go, Python) and
  pay-kit's verifiers.
- RPC path promoted from reported to gated: every vector through `simulateTransaction` and
  `sendTransaction` against `solana-test-validator`, with the RPC's answer as a report column.
- Acceptance: at least 15 testees; RPC column; every signer finding disclosed and documented.

### M3 (weeks 12-16, USD 10,000): second judge, adoption, handover

- Firedancer as a second judge: the vector set through Firedancer's transaction parser; any
  Agave-vs-Firedancer acceptance difference reported to both teams privately.
- Vectors adopted upstream: PRs adding the fixtures to `@solana/kit`, `solders` and
  `solana-go` test suites so the suite runs in their CI.
- Public results page regenerated by the weekly run: every testee, every release tested,
  every open divergence with the vector that reproduces it.
- "SIMD-0385 implementer's checklist" derived from the vector classes, offered to the Larger
  Transaction Sizes docs page.
- Handover: contributor guide, release process for new vector versions when a SIMD changes
  the format, 12 months of weekly runs after M3.
- Acceptance: Firedancer column; at least two upstream fixture PRs merged or in review;
  results page live.

### Budget

| Item | USD |
|---|---|
| M1, 5 weeks, about 25 h/week | 8,000 |
| M2, 6 weeks (emulators, four firmware parsers, payment stacks, RPC gating) | 12,000 |
| M3, 5 weeks (Firedancer build, upstream fixtures, results page, docs) | 10,000 |
| Infrastructure (CI minutes for emulators and a validator; results page hosting) | included |
| Total | 30,000 |

M1 alone (USD 8,000) is self-contained if the Foundation prefers a smaller first commitment.

## Team

Ashwin Goyal, B.Tech CSE at VIT Bhopal (2024-2028), Navi Mumbai. GitHub
`let-the-dreamers-rise` (147 repositories). Portfolio: https://ashwin-goyal-portfolio.vercel.app/.

Exactly this technique, with fixes merged by the maintainers:
- `bitcoin/bips` PR #2273 (merged, Sep 2026): a spec-only differential suite found two bugs in
  the BIP-360 reference implementation (unbounded Merkle depth producing consensus-invalid
  outputs; asserts stripped under `python -O`). Companion repo `p2mr-assurance-lab`: four
  graded divergences against the official reference, two of them real gaps.
- `x402-foundation/x402` PR #3674 (merged, Oct 2026, +195/-4): the Cardano client
  re-serialized signed transactions through a non-byte-preserving round-trip so nodes
  rejected valid payments; fix submits the signed bytes unchanged. Same bug class as F1 here.
- txconform itself, built and run against eight implementations before this application.

Solana work: proofflow (Anchor programs, Arcium MPC, Solana Attestation Service, USDC/CCTP),
squad-treasury (policy-gated treasury agent with Solana payouts and x402 purchases),
genesis-solana-agent (Colosseum Agent Hackathon). Earlier grants: 0G Buildathon, Genkai
Agentic Superengineering. Finalist, Meta PyTorch Hackathon.

AI tools were used to draft code and documents. Every vector verdict comes from running
Agave's crates, every SDK result from running that SDK, and every finding was reproduced by
hand before being written down. The same holds for the grant work.

## Risks

- A maintainer may call a divergence intentional (trailing bytes, for example). The suite
  records it as a documented divergence either way, which is what integrators need.
- Hardware-wallet emulators are slow; M2 has six weeks for four of them, and Speculos and
  the Trezor emulator are open source and scriptable.
- The format may change (a SIMD defining a mask bit). The weekly run, versioned vectors and
  M1's forward-compatibility set exist for that.
- Solo maintainer. The vectors are JSON under Apache-2.0; M3 moves them into the SDKs' own CI
  so they outlive the author.
