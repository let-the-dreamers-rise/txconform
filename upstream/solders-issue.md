Post to: https://github.com/kevinheavey/solders/issues/new

Title: TransactionConfig docstring says priority_fee is in micro-lamports; SIMD-0385 defines it as total lamports

`solders.message.TransactionConfig.__doc__` (0.29.0):

> priority_fee (Optional[int]): The priority fee in micro-lamports.

SIMD-0385 defines bits 0–1 of the config mask as "Total priority-fee lamports" (u64), and
Agave's `TransactionConfigView::priority_fee_lamports` and the kit/web3.js field
`priorityFeeLamports` agree. The wire bytes solders writes are correct; only the
documentation is wrong, but a Python developer porting a v0 fee calculation (micro-lamports
per CU) will pay 10^6 times the intended fee or compute the wrong total.

Suggested: fix the docstring and consider a `priority_fee_lamports` alias. Found while
running solders against https://github.com/let-the-dreamers-rise/txconform; solders
otherwise matched Agave on 456 of 461 vectors (the 5 differences are trailing-bytes
acceptance and size limits, which the Rust SDK layer it wraps also does not enforce).
