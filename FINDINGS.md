# Findings (1 November 2026)

Each finding lists the vector class to reproduce it (`vectors/vectors.jsonl`, regenerate
with `./run.sh`), what Agave does, what the SDK does, and why it matters. Severity is my
assessment; maintainers may disagree. Nothing here is a fund-loss bug. The common thread
is that each SDK re-derived SIMD-0385 on its own and no two agree with the validator at the
edges.

## F1. `@solana/kit` 8.6.0 accepts reserved config-mask bits that Agave rejects

- Vectors: `v1-mask-unknown-bit` (bit 5 set, no value word), `v1-mask-unknown-bit-with-value`.
- Agave: `agave-transaction-view` rejects any mask with bits outside `0b1_1111`
  (`transaction_config_frame.rs`, `sanitize_mask`), and the Rust SDK's wincode reader rejects
  it too with the comment that unknown bits "would be silently dropped on re-serialization
  (this message is signed, so that invalidates the signature)".
- kit: `getCompiledTransactionMessageDecoder` decodes the message, reports `configMask: 36`,
  reads values only for the bits it knows, and `decompileTransactionMessage` succeeds.
  With the value word present (the layout a future SIMD would use), decoding fails with the
  unrelated error `Codec [u8] cannot decode empty byte arrays` because the instruction
  headers are read from the wrong offset.
- `@solana/web3.js` 3.1.1, built on kit, does reject: `Unexpected bits set in the transaction
  config mask`. So the check exists one layer up but not in the codec most wallets will call.
- Why it matters: a signer that decodes with kit, shows the four known fields, and signs
  has signed over configuration it did not display. Today no such bit is defined, so this is
  forward-compatibility hygiene, but it is exactly the class the Agave comment warns about.
- Suggested fix: reject `configMask & ~0b11111` and odd priority-fee pairs in the compiled
  message decoder; size `configValues` by `popcount(mask)`.

## F2. Every SDK accepts trailing bytes and an extra trailing signature; the validator does not

- Vectors: `v1-trailing-byte`, `v1-extra-signature` (64 extra bytes after the message).
- Agave: `TransactionFrame::try_new` requires the frame to end exactly at the buffer end
  (`ParseError`).
- kit 8.6.0, web3.js 3.1.1, solders 0.29.0 and the Rust SDK `wincode::deserialize` all decode
  successfully and report the header-specified number of signatures. solana-go 2.2.0
  rejects both. web3.js 1.99.0 rejects the trailing byte but accepts the extra signature.
- Why it matters: anything that verifies a client-supplied transaction before relaying it
  (x402 facilitators, paymasters, co-signers, bundlers) can accept bytes that no validator
  will ever include. The transaction simply never lands, which is an availability and
  accounting problem rather than theft, but it is a divergence that should be documented
  or closed (`createDecoderThatConsumesEntireByteArray` exists in kit for this purpose).

## F3. `solana-go` v2.2.0 `Sanitize()` misses size limits and legacy/v0 duplicates

- Vectors: `legacy-size-over-1232` (1,235 bytes), `v1-size-4097`, `legacy-duplicate-address`.
- Agave: 1,232-byte limit for legacy/v0 is enforced in the view sanitizer; 4,096 for v1 and
  duplicate addresses in `solana-runtime-transaction`.
- solana-go: decodes and `Sanitize()` returns nil for all three. Its `sanitizeV1` checks
  duplicates, but `sanitizeLegacy` and `sanitizeV0` do not, and no path checks the serialized
  size. (It is otherwise the strictest SDK tested: it rejects 13 signatures, 65 addresses,
  heap bounds and trailing bytes, which the Rust SDK itself does not.)
- Suggested fix: add `len(data) <= 1232 / 4096` by version in `TransactionFromBytes`, and a
  duplicate check for legacy/v0 static keys.

## F4. `@solana/web3.js` 1.99.0 fails to read valid v1 transactions with a priority fee above 2^53

- Vectors: `v1-priority-fee-only`, `v1-config-max-values`, 97 of 200 `v1-random`.
- Agave accepts any u64 priority fee at the format level (the fee is simply charged).
- web3.js 1.x `decodeU64` asserts `value <= Number.MAX_SAFE_INTEGER`
  (`lib/index.cjs.js:1172`) and throws `Expected u64 value to be within the safe integer
  range`. The 3.x line uses BigInt and is unaffected.
- Why it matters: a block reader or indexer on web3.js 1.x throws on such a transaction.
  A fee above 9,007 TSOL cannot actually be paid, so this is robustness, not exploitability;
  it is still a valid-rejected conformance failure. Also confirmed: 1.x cannot serialize v1
  (`Serialization of version 1 transaction messages is not supported`), which is documented
  read-only support, and it names the field `priorityFee` where 3.x says `priorityFeeLamports`.

## F5. `solders` 0.29.0 documents the v1 priority fee in micro-lamports

- `TransactionConfig.__doc__`: "priority_fee (Optional[int]): The priority fee in
  micro-lamports."
- SIMD-0385 and Agave: the field is the **total** priority fee in **lamports**. Every other
  SDK tested names it `priority_fee_lamports`/`priorityFeeLamports`. The wire value solders
  produces is correct; the documentation tells a Python developer to supply a number that is
  one million times too large (or, reading it as per-CU, to compute the wrong total).
- Suggested fix: one-line docstring change plus a rename alias `priority_fee_lamports`.

## F6. The Rust SDK's `sanitize()` is not the validator's acceptance test (layer map)

Not a bug, but the fact that makes shared vectors necessary. `solana-transaction` 6.0.0
`VersionedTransaction::sanitize` accepts 13 signatures, 65 addresses, 65 instructions,
out-of-range heap sizes, 4,097 bytes, trailing bytes and duplicate addresses; its own test
is named `v1_sanitize_accepts_counts_above_old_policy_limits`. Those limits now live in
`solana-runtime-transaction`. Ports that "mirror the SDK" (solana-go's comments say
exactly that) inherit whichever layer they copied. The oracle records all three layers so
that SDK authors can see which rule they are missing and where Agave keeps it. Run as a
testee in its own right (`tools/rust_sdk_results.py`), the Rust SDK accepts 13 of the 47
Agave-invalid vectors, more than solders (5) or solana-go (3).

## F7. `gagliardetto/solana-go` v1.25.0 has exactly the three gaps of `solana-go` v2

- Vectors: `legacy-size-over-1232`, `v1-size-4097`, `legacy-duplicate-address`.
- Both Go SDKs decode and `Sanitize()` all three; both reject everything else Agave rejects
  (13 signatures, 65 addresses, heap bounds, trailing bytes). The identical signature across
  2,400 mutation vectors suggests shared parsing code; whichever is fixed first, the vectors
  tell the other where to look.

## F8. `solana-primitives` 0.3.0 is the strictest implementation tested; one gap

- Only `legacy-duplicate-address` is accepted (Agave: `AccountLoadedTwice` in the runtime).
  Everything else, including every mutation vector, matches Agave. Its `deserialize()`
  enforces exact consumption, size by version, mask bits and sanitize in one call, which is
  the API shape the other SDKs lack.

## F9. Agave's own RPC accepts transactions its block producer drops: a signature is returned and nothing lands

Measured end to end in CI on `solana-test-validator` from Agave 4.4.0 (`harness/rpc/send_probe.py`,
artifact `txconform-rpc-results`, run 41279892753):

| transaction | `sendTransaction` (preflight on) | `sendTransaction` (preflight off) | landed within 20 s |
|---|---|---|---|
| legacy transfer, clean | accepted, signature returned | accepted | yes (slot 118 / 119) |
| legacy transfer + 1 trailing byte | **accepted, signature returned** (simulation ran and passed) | accepted | **no** |
| legacy transfer + extra signature | rejected: failed to deserialize | rejected | n/a |
| v1 transfer, clean | simulation ran (failed on a budget cap I set too low) | accepted | yes (slot 21) |
| v1 transfer + 1 trailing byte | simulation ran (same budget error, so the format was accepted) | **accepted, signature returned** | **no** |
| v1 transfer + extra signature | simulation ran | **accepted, signature returned** | **no** |

What happens: `sendTransaction` and `simulateTransaction` deserialize with the SDK path
(`solana_transaction::versioned::VersionedTransaction`, the error strings name it), which
tolerates trailing bytes (F2 and F6). The RPC then forwards the original wire bytes to the TPU,
where `agave-transaction-view` requires the frame to end at the buffer and drops the packet.
The client sees a signature and "preflight passed"; `getSignatureStatuses` never returns a
status; no error is ever produced. Across the 3,012-vector corpus the RPC path accepted 4
Agave-invalid vectors, all of them trailing-byte shapes (`v1-trailing-byte`, `v1-extra-signature`
and two mutations that shorten the message so bytes remain).

Why it matters: any client whose SDK can emit trailing bytes (an extra signature slot, a
re-serialization bug, a buffer not trimmed) gets a silent, retry-forever failure instead of an
error, and a facilitator that "verifies by simulating" approves it. This is not fund loss; it
is an availability and accounting hole inside the node itself, and it is exactly the layer
disagreement that makes a validator-judged vector set necessary. Suggested fix (for Anza):
reject trailing bytes in the RPC's `decode_and_deserialize`, or sanitize with the transaction
view before accepting; draft in `upstream/agave-issue.md`.

## Mutation corpus: what 2,500 bit flips and truncations added

`gen --mutations 1000` adds every truncation point of a v1 and a legacy template (about
560 vectors), every single-bit flip of the first 48 bytes of a full-config v1 template (384)
and 1,000 random single-bit flips over v1, v1-with-config, legacy and v0 templates. Agave
judges each one; the spec does not say what a flipped bit should do, so the Agave verdict is
the only expectation.

Result: every accepts-invalid vector in the mutation classes maps to a root cause already
named above. Flipping a config-mask bit or `num_addresses` shifts the layout so the message
ends early; Agave's packet parser rejects (`ParseError`, frame must end at the buffer), while
the Rust SDK, kit, web3.js and solders decode the shorter message and ignore the rest (F2).
Flips inside a config word push `heap_size` out of range (F6 layer). Flips in instruction
account indexes make them out of bounds (kit and web3.js do not check indexes). web3.js 1.x
throws on 327 flipped-fee vectors (F4). No new root cause appeared, which is some evidence
that the 41 hand-written invalid classes cover the format's failure modes; the corpus stays
in CI so that a future SDK release is measured against 3,000 vectors, not 460.

## Spec notes surfaced by the generator

- SIMD-0385's constraint table says 64 addresses, 12 signatures and 64 instructions; the
  wire format allows 96, 42 and 255. Agave enforces the smaller numbers in the runtime
  layer, and none of the four non-Rust SDKs except solana-go enforce them anywhere.
- A ComputeBudget instruction inside a v1 transaction is a valid no-op
  (`v1-computebudget-ix-ignored`). All eight testees correctly report the config from the mask
  and not from the instruction, which is the behaviour the Larger Transaction Sizes guide
  asks wallets for.
- Unset config fields have runtime defaults of 0 / 0 / 0 / 32 KiB. web3.js 1.x materialises
  the defaults; kit, web3.js 3.x, solders and solana-go report absence. The diff tool treats
  both as equal.

## Not yet covered (proposed work)

Hardware-wallet and browser parsers (Ledger `app-solana` C, once #248 lands; Trezor firmware
Python; Keystone Rust; Brave C++), x402's three SVM mechanism implementations, Java (`sava`),
Kotlin (`web3-core`), Swift, Dart and C# SDKs, and the RPC path (`sendTransaction` /
`simulateTransaction` behaviour per vector against a local validator). The vectors are
language-neutral JSON; each of these needs only a 100-line harness.
