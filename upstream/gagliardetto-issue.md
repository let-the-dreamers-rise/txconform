Post to: https://github.com/gagliardetto/solana-go/issues/new

Title: Sanitize() does not enforce serialized size limits or legacy duplicate addresses

Hi, I run txconform (https://github.com/let-the-dreamers-rise/txconform), a set of transaction wire vectors judged by Agave's own parser crates (`agave-transaction-view`, `solana-runtime-transaction`), with a small harness per SDK. On v1.25.0, three vector classes decode and pass `Sanitize()` although a validator rejects them:

| vector class | bytes | Agave | solana-go |
|---|---|---|---|
| `legacy-size-over-1232` | legacy tx of 1,235 bytes | rejected in the packet view (1,232-byte limit) | decodes, Sanitize ok |
| `v1-size-4097` | v1 tx of 4,097 bytes | rejected (`SanitizeFailure`, 4,096-byte limit) | decodes, Sanitize ok |
| `legacy-duplicate-address` | static key repeated | rejected at runtime (`AccountLoadedTwice`) | decodes, Sanitize ok |

Everything else Agave rejects (13 signatures, 65 addresses, 65 instructions, heap bounds, trailing bytes, reserved mask bits) is rejected here too, which makes the library one of the stricter ones tested. The same three gaps appear in solana-foundation/solana-go v2.2.0, so the code may be shared.

Suggested fix: check `len(data)` against 1232 / 4096 by version in `TransactionFromBytes`, and reject repeated static keys in legacy and v0 sanitize (v1 already does).

Reproduce: `./run.sh` in the repo regenerates the vectors; the harness is `harness/go-gagliardetto/main.go` (about 100 lines). Happy to open a PR if you prefer.
