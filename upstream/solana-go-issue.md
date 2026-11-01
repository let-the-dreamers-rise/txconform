Post to: https://github.com/solana-foundation/solana-go/issues/new

Title: Sanitize() does not enforce serialized size limits or legacy/v0 duplicate addresses

`TransactionFromBytes` + `Sanitize()` return nil for:

1. a legacy transaction of 1,235 bytes (Agave: `PACKET_DATA_SIZE` 1232, rejected in
   `agave-transaction-view` sanitize);
2. a v1 transaction of 4,097 bytes (Agave: `v1::MAX_TRANSACTION_SIZE` 4096, rejected in
   `solana-runtime-transaction`);
3. a legacy transaction with a duplicated static address (Agave: `AccountLoadedTwice`).
   `sanitizeV1` calls `HasDuplicates()` but `sanitizeLegacy`/`sanitizeV0` do not.

Vectors `legacy-size-over-1232`, `v1-size-4097`, `legacy-duplicate-address` in
https://github.com/let-the-dreamers-rise/txconform (bytes attached). Otherwise solana-go
was the strictest SDK of the five I tested: it is the only one that rejects 13 signatures,
65 addresses, heap out of range and trailing bytes, matching the validator.

Suggested fix: check `len(data)` against 1232 / 4096 by version in `TransactionFromBytes`
(or `Sanitize`), and run the duplicate check for all versions. I can send a PR.
