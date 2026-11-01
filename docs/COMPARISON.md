# Comparison with what the Solana Foundation has funded

Written 1 Nov 2026. The Foundation publishes grantee names and some RFP amounts but not
applications, so this compares against the public RFP deliverable specs, two Foundation-funded
tooling projects whose state at funding is documented, and the Foundation's own in-house
conformance work. Ash's rule: match or beat every detail.

## Reference points

| Reference | Amount | What was funded | State at funding (as far as public) | Source |
|---|---|---|---|---|
| RFP: Codama Python renderer | $30,000 | A renderer generating Python clients from Codama IDL | Spec'd by the Foundation; applicant delivers | blog.colosseum.com Solana RFPs post |
| RFP: Math library (fixed-point / big-number for SBF) | $45,000 | Audited math crate | Same | same |
| RFP: Historical state verification | $275,000 | Research + implementation | Same | mpost.io RFP coverage |
| RFP: Indexer tooling + discriminator DB | $160,000 (two RFPs) | Open-source data tooling | Same | same |
| Trident (Ackee Blockchain) | Foundation grant, amount not public | Fuzzing framework for Anchor programs | Existing audit firm, working prototype | Ackee blog / Trident README |
| Mithril (Overclock) | Foundation grant, amount not public | Go full-node verifier (client diversity) | Validator operator team, design doc | Overclock announcement |
| pay-kit conformance harness (in-house, L. Galabru) | Foundation staff time | 619 protocol vectors, 9 SDK testees, rulings doc, CI matrix | Built Oct 2026 inside the Foundation | github.com/solana-foundation/pay-kit/pull/387 |

Observations:
1. Tooling the Foundation pays for is **spec'd as concrete deliverables** (renderer, crate, harness), not as "we will research".
2. **Client diversity and conformance are already Foundation priorities**: Mithril (second validator client), Firedancer conformance, and now pay-kit's harness built by Foundation staff. txconform is the same idea pointed at the layer nobody covers: client SDKs and wallets.
3. Grantees had a **working prototype or a prior track record in the exact technique**.

## Where txconform stands against each detail

| Detail past grantees showed | txconform at application time |
|---|---|
| Working code | 461 vectors generated, 6 testees run (5 SDKs plus the Rust SDK itself), full pipeline in 5 s on a cached build (`run.sh`), CI workflow, baseline gate |
| Concrete findings | 5 divergence classes across 5 SDKs plus the Rust SDK's own laxness (FINDINGS.md F1-F6), each with the vector class that reproduces it, Agave's behaviour and the SDK's behaviour, plus 4 upstream issue drafts ready to post |
| Validator-grade oracle | Agave's own crates (`agave-transaction-view` 9.0, `solana-runtime-transaction` 4.5, `solana-transaction` 6.0) run per vector, verdict recorded per layer. Generator's spec reading agreed with Agave on 461/461, so the spec text and the implementation were cross-checked too |
| Cross-language | TypeScript (kit, web3.js 3.x, web3.js 1.x), Python (solders), Go (solana-go v2), Rust (SDK) today; vectors are language-neutral JSON |
| Fits the Foundation's own work | Explicit plan to ship an adapter into pay-kit's `harness/vectors` format and to upstream vectors into kit's test fixtures |
| Track record in the technique | Ash's merged `bitcoin/bips` PR #2273 (spec-only differential suite found two reference-implementation bugs in BIP-360) and merged `x402-foundation/x402` PR #3674 (byte-preserving transaction submission fix on Cardano; the same class of "re-serialization changes signed bytes" bug that F1 warns about on Solana) |
| Prior grants | 0G Buildathon ($67), Genkai Agentic Superengineering ($200) |
| License | Apache-2.0 |
| Budget shape | Three milestones, each with a checkable deliverable, $30k total; smaller than the Foundation's smallest public RFP ($30k) relative to scope |

## What past grantees had that txconform does not

- A company or audit-firm name (Ackee, Overclock). txconform is a solo student. Mitigation: the code already runs and found bugs; milestones are deliverable-gated; the Foundation can stop after M1.
- Direct relationships with Foundation engineers. Mitigation: open the upstream issues before submitting so reviewers can see maintainers' responses, and offer the pay-kit adapter first.
- Public Solana on-chain footprint (the form asks for Solana accounts). Ash has none yet. Mitigation: state it plainly; the deliverable is off-chain tooling.
