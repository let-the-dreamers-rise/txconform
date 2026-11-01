Post to: https://github.com/anza-xyz/agave/issues/new (or security@anza.xyz first if you prefer; this is an availability issue, not fund loss)

Title: RPC sendTransaction accepts transactions with trailing bytes that the TPU drops; client gets a signature and the transaction never lands

**Summary.** `sendTransaction` (and `simulateTransaction` preflight) deserialize the wire bytes with `solana_transaction::versioned::VersionedTransaction`, which accepts trailing bytes after the last signature. The RPC returns a signature and forwards the original bytes; `agave-transaction-view` in the TPU requires the frame to end exactly at the buffer and drops the packet. The client never gets an error and `getSignatureStatuses` stays null forever.

**Reproduction** (Agave 4.4.0, `solana-test-validator --reset`), script: https://github.com/let-the-dreamers-rise/txconform/blob/main/harness/rpc/send_probe.py

1. Airdrop to a fresh keypair; build and sign a System transfer (legacy or v1).
2. Send the clean bytes: accepted and lands (slot N).
3. Append one `0x00` byte and send with `skipPreflight: false`: preflight simulation passes, a signature is returned, the transaction never lands. Same with `skipPreflight: true`. Same for a v1 transaction with 64 extra bytes after the signatures.
4. A legacy transaction with an extra signature *is* rejected at deserialize (the compact-u16 count disagrees with the header), so the gap is specifically "bytes after a well-formed transaction".

Results table from a CI run (artifact `txconform-rpc-results`, run 41279892753) is in the repo's FINDINGS.md (F9).

**Expected.** Either the RPC rejects the bytes at `decode_and_deserialize` (`-32602`), the same way it already rejects oversize payloads, or it sanitizes through the transaction view before accepting. The view already exists in the TPU; the RPC path could call `SanitizedTransactionView::try_new_sanitized` on the decoded bytes.

**Context.** This came out of txconform, a set of transaction wire vectors judged by Agave's own parser crates and run against client SDKs. kit, web3.js 3.x, solders and the Rust SDK also accept trailing bytes (they mirror the SDK deserializer), so a client bug that leaves bytes behind passes every layer until the TPU, silently. Happy to provide the raw vectors or a PR.
