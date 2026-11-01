Post to: https://github.com/anza-xyz/kit/issues/new

Title: Compiled transaction message decoder accepts reserved config-mask bits (v1 / SIMD-0385)

`getCompiledTransactionMessageDecoder()` decodes a v1 message whose `TransactionConfigMask`
has a bit outside `0b11111` set, reads values only for the known bits, and
`decompileTransactionMessage` then succeeds. Agave rejects such a message at the packet
parser (`agave-transaction-view`, `sanitize_mask`) and in `solana-message`'s wincode reader,
whose comment explains why: unknown bits would be silently dropped on re-serialization and
the message is signed. `@solana/web3.js` 3.x on top of kit does throw
`Unexpected bits set in the transaction config mask`, so the check exists one layer up.

Repro (bytes attached, vector `v1-mask-unknown-bit` from
https://github.com/let-the-dreamers-rise/txconform):

```js
const cm = getCompiledTransactionMessageDecoder().decode(messageBytes); // configMask: 36
decompileTransactionMessage(cm); // ok
```

With the value word that an unknown bit would carry (`v1-mask-unknown-bit-with-value`),
decoding fails later with `Codec [u8] cannot decode empty byte arrays` because the
instruction headers are read from a shifted offset.

Suggested: reject `configMask & ~0b11111 !== 0` and `(mask & 0b11) in {1, 2}` in the
decoder, and size `configValues` by `popcount(mask)` so the failure is explicit. Related:
`getTransactionDecoder` also accepts trailing bytes and an extra trailing signature that
the validator rejects (`v1-trailing-byte`, `v1-extra-signature`); happy to open that
separately if you want it tracked.

Found with txconform, a spec-derived vector set judged by Agave's own parser; 461 vectors,
report at results/REPORT.md.
