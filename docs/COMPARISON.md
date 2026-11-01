# Comparison with what the Solana Foundation has funded, line by line

Written 1 Nov 2026 (v2, after Ash asked for weeks-of-production quality). The Foundation
publishes grantee names and RFP amounts but not applications, so "their pitch" below is
reconstructed from the grantee's own announcement or the Foundation's RFP text, each
fetched and quoted today. Ash's rule: match or beat every detail, and keep this note.

## 1. The grantees and how they pitched

| Grantee | Funded by | What they had when pitched | How they framed value | Numbers in the pitch | Source |
|---|---|---|---|---|---|
| **Trident** (Ackee Blockchain), Anchor fuzzing framework | Solana Foundation development grant, 2023 | Working tool (born as Trdelnik, Riptide hackathon 2022, Marinade community prize); an audit firm behind it | "The First Open-Source Fuzzer for Solana Programs"; fuzzing finds "unknown bugs and vulnerabilities, potentially preventing zero-day exploits" | None: no users, no bug counts, no benchmarks | ackee.xyz blog "Introducing Trident"; Colosseum Codex |
| **sec3 / Soteria** (Coderrect), static analysis for Solana programs | Solana Foundation grant, Dec 2021 | Existing company with "fundings from top organizations", Fortune 500 customers, audit services | "In the race towards Web3, security is a grand challenge" | None | sec3.dev press release |
| **Mithril** (Overclock), Go verifying client | Foundation-backed client-diversity work (amount not public) | Validator-operator team; alpha that relies on `getBlock` and can halt on mismatches; external audit under way | "advances client diversity", "lowers the cost of running validators" | None in the Codex write-up | Colosseum Codex 23 Jan 2026 |
| **RFP: Codama Python renderer** | $30,000 RFP | Spec written by the Foundation | Deliverable-defined | Amount only | Colosseum RFP post |
| **RFP: Math library** | $45,000 RFP | Same | Same | Amount only | same |
| **RFP: Indexer tooling + discriminator DB** | $160,000 (two RFPs) | Same | Same | Amount only | mpost coverage |
| **RFP: Historical state verification** | $275,000 RFP | Same | Same | Amount only | same |
| **pay-kit conformance harness** (built in-house by Foundation staff, PR #387, 21 Oct 2026) | Staff time | 619 vectors, 9 SDK testees, rulings document, CI matrix, nightly variant corpus | "prove those independent implementations actually agree with each other" | Vector counts per intent; per-SDK pass / known-failing counts | pay-kit harness README, PR #387 |
| Wave-one grantees (2020): Wormhole, WalletConnect, Orca, Validators.app... | Early ecosystem grants | Teams with products or well-known brands | "talented team and a strong vision" | None public | solana.com news, Nov 2020 |

Patterns, stated plainly:
1. **Funded tooling had a working thing and a specific gap claim.** Trident's gap claim was
   a headline with no prior-art check; it was funded anyway because the tool existed.
2. **Numbers were rare.** None of the three tooling pitches gave measured results. The only
   number-dense artefact is the Foundation's own harness, which reports counts per SDK.
3. **The Foundation now builds conformance tooling itself** (pay-kit harness). That is both
   the risk ("we do this in-house") and the proof that they value it.
4. **Deliverables beat vision.** Every RFP is a deliverable spec with a price.

## 2. Line by line: theirs vs txconform

| Detail | Best past grantee | txconform at application (1 Nov 2026) | Beats? |
|---|---|---|---|
| Working code | Trident: tool existed for 2 years | Oracle, generator, mutation corpus, 8 harnesses, diff, CI, baseline gate, pay-kit export, RPC job. Repo public, CI green | Match (younger, but complete pipeline) |
| Measured results in the pitch | pay-kit harness: counts per SDK | 3,012 Agave-judged vectors, 63 classes, 8 testees, per-testee divergence counts, 8 findings with the vector that reproduces each, 15 s pipeline | Beat (only artefact with validator-judged numbers) |
| Gap claim with prior-art check | Trident: none; sec3: none | PRIOR-ART.md: 9 rows, including the Foundation's own harness and why it does not cover this | Beat |
| Judge | pay-kit: its own Rust implementation; solana-primitives: the Rust SDK | Agave's crates, three layers, verdict recorded per layer; cross-checked against spec reading (0 disagreements on hand-written vectors) | Beat |
| Breadth of testees | pay-kit: 9 languages of one product | 8 independent implementations in 4 languages, including both Go SDKs, the Rust SDK and an independent Rust crate; plan for Java, Kotlin, Swift, Dart, C#, firmware | Match now, beat at M1 |
| Findings landed upstream | Trident/sec3: n/a | 6 issue drafts ready (kit, web3.js 1.x, solders, solana-go v2, gagliardetto, solana-primitives); none posted yet (Ash's click) | Behind until posted |
| Fits Foundation's current direction | Mithril: client diversity | Same idea one layer down; explicit pay-kit adapter in the plan and already exported | Match |
| Team credibility | Ackee (audit firm), Coderrect (company), Overclock (validator) | Solo student; merged `bitcoin/bips` #2273 and `x402` #3674; Solana work: proofflow (Anchor, Arcium, SAS, CCTP), squad-treasury (Solana payouts, x402), genesis-solana-agent (Colosseum Agent Hackathon), p2mr-assurance-lab (BIP-360 conformance, 4 graded divergences) | Behind on brand, ahead on exactly-this-technique track record |
| License | Apache-2.0 / MIT | Apache-2.0 | Match |
| Budget shape | RFPs: fixed deliverable, $30k-$275k | $30k, three deliverable-gated milestones, M1 standalone | Match (smallest public RFP tier) |
| Public-good durability | Mithril: a client others run | Vectors are JSON; M3 puts them into kit, solders, solana-go CI so they run without the author | Match |

## 3. What this comparison changed in the proposal

- Opening paragraph now answers the two rejection reasons first (in-house/AI; already
  covered), because the Foundation's own harness is the obvious objection.
- Replaced "five SDKs" with the measured eight-testee table and the 3,012-vector corpus.
- Added the pay-kit export as a shipped artefact, not a promise.
- Team section uses Ash's Solana repos (proofflow, squad-treasury, genesis-solana-agent) and
  conformance repos (p2mr-assurance-lab), never Rein.
