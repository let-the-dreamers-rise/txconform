Post to: https://github.com/solana-foundation/solana-web3.js/issues/new (legacy 1.x line)

Title: MessageV1.deserialize throws on valid v1 transactions whose priority fee exceeds 2^53-1

`@solana/web3.js` 1.99.0 added read-only v1 support. `decodeU64` asserts
`value <= Number.MAX_SAFE_INTEGER` (`lib/index.cjs.js:1172`), so any v1 transaction with
`priorityFee > 9007199254740991` lamports fails with
`Expected u64 value to be within the safe integer range`, even though the format is valid
and Agave reads it. 97 of 200 random spec-valid vectors in
https://github.com/let-the-dreamers-rise/txconform hit this. Such a fee cannot realistically
be paid, so impact is a block reader/indexer throwing rather than anything worse, but
`getTransaction`/`getBlock` consumers on 1.x would still see an exception. 3.x uses BigInt
and is fine.

Suggested: return a BigInt for this field (as 3.x does with `priorityFeeLamports`), or clamp
with a documented flag. Also noting for consistency: 1.x names the field `priorityFee`,
3.x `priorityFeeLamports`.
