Post to: https://github.com/hewigovens/solana-primitives/issues/new

Title: deserialize() accepts a legacy transaction with a duplicated static account key

Hi, I run txconform (https://github.com/let-the-dreamers-rise/txconform): transaction wire vectors judged by Agave's own parser crates, with a harness per SDK. solana-primitives 0.3.0 is the strictest implementation of the eight tested (it matches Agave on 3,011 of 3,012 vectors, including every mutation). The one difference:

- `legacy-duplicate-address`: a legacy transaction whose static account keys contain the same pubkey twice. `VersionedTransaction::deserialize` returns Ok and `sanitize()` passes. Agave rejects it in the runtime (`AccountLoadedTwice`, `solana-runtime-transaction`). The Rust SDK's own `sanitize()` also accepts it (the check lives in the runtime crate, not the SDK), so this is a case where mirroring the SDK inherits the SDK's gap.

Suggested fix: reject repeated static keys in legacy and v0 sanitize (v1 already rejects duplicates).

Vectors are plain JSON (hex bytes plus Agave's per-layer verdict); the harness is `harness/rust-primitives/src/main.rs`. Happy to send a PR.
