# txconform report

Vectors: 3012 (oracle = Agave: agave-transaction-view + solana-runtime-transaction + solana-transaction)

| harness | valid vectors | invalid vectors | rejects-valid | accepts-invalid | field-mismatch | roundtrip |
|---|---|---|---|---|---|---|
| solana-transaction@6.0.0 | 1681 | 1331 | 0 | 17 | 0 | 0 |
| @solana/kit@8.6.0 | 1681 | 1331 | 0 | 89 | 0 | 0 |
| @solana/web3.js@3.1.1 | 1681 | 1331 | 0 | 58 | 0 | 0 |
| @solana/web3.js@1.99.0 | 1681 | 1331 | 327 | 48 | 0 | 658 |
| solders@0.29.0 | 1681 | 1331 | 0 | 7 | 0 | 0 |
| solana-go/v2@v2.2.0 | 1681 | 1331 | 0 | 3 | 0 | 0 |
| gagliardetto/solana-go@v1.25.0 | 1681 | 1331 | 0 | 3 | 0 | 0 |
| solana-primitives@0.3.0 | 1681 | 1331 | 0 | 1 | 0 | 0 |

## solana-transaction@6.0.0

### ACCEPTS-INVALID: 17 vectors in 15 classes

| class | n | example | detail |
|---|---|---|---|
| `legacy-duplicate-address` | 1 | duplicate static address in a legacy tx (static sanitize passes; runtime rejects) | runtime_resolved: AccountLoadedTwice |
| `legacy-size-over-1232` | 1 | legacy transaction of 1235 bytes (> 1232) | view_sanitize: SanitizeError |
| `mut-bitflip-v1-config` | 3 | v1-config template, byte 157 bit 1 flipped | runtime_static: SanitizeFailure |
| `mut-bitflip-v1-header` | 1 | v1-config template, byte 40 bit 1 flipped | view_parse: ParseError |
| `v1-13-signatures` | 1 | 13 signatures > max 12 | runtime_static: SanitizeFailure |
| `v1-65-addresses` | 1 | 65 addresses > max 64 | runtime_static: SanitizeFailure |
| `v1-65-instructions` | 1 | 65 instructions > max 64 | runtime_static: SanitizeFailure |
| `v1-duplicate-address` | 1 | address 1 duplicates address 0 | runtime_resolved: AccountLoadedTwice |
| `v1-extra-signature` | 1 | one more signature than num_required_signatures (trailing 64 bytes) | view_parse: ParseError |
| `v1-heap-above-max` | 1 | heap 257 KiB > 256 KiB | runtime_static: SanitizeFailure |
| `v1-heap-below-min` | 1 | heap 31 KiB < 32 KiB | runtime_static: SanitizeFailure |
| `v1-heap-not-1k-multiple` | 1 | heap 32769 not a multiple of 1024 | runtime_static: SanitizeFailure |
| `v1-heap-zero` | 1 | heap bit set with value 0 | runtime_static: SanitizeFailure |
| `v1-size-4097` | 1 | 4097 bytes > max 4096 | runtime_static: SanitizeFailure |
| `v1-trailing-byte` | 1 | one extra byte after the last signature | view_parse: ParseError |

## @solana/kit@8.6.0

### ACCEPTS-INVALID: 89 vectors in 29 classes

| class | n | example | detail |
|---|---|---|---|
| `legacy-duplicate-address` | 1 | duplicate static address in a legacy tx (static sanitize passes; runtime rejects) | runtime_resolved: AccountLoadedTwice |
| `legacy-program-is-fee-payer` | 1 | program_id_index = 0 | view_sanitize: SanitizeError |
| `legacy-size-over-1232` | 1 | legacy transaction of 1235 bytes (> 1232) | view_sanitize: SanitizeError |
| `mut-bitflip-legacy` | 1 | legacy template, byte 259 bit 4 flipped | view_sanitize: SanitizeError |
| `mut-bitflip-v0` | 2 | v0 template, byte 854 bit 7 flipped | view_sanitize: SanitizeError |
| `mut-bitflip-v1` | 8 | v1 template, byte 158 bit 7 flipped | view_sanitize: SanitizeError |
| `mut-bitflip-v1-config` | 11 | v1-config template, byte 157 bit 1 flipped | runtime_static: SanitizeFailure |
| `mut-bitflip-v1-header` | 43 | v1-config template, byte 1 bit 1 flipped | view_parse: ParseError |
| `mut-truncate-legacy` | 1 | legacy template cut to 805 of 806 bytes | view_parse: ParseError |
| `v0-lookup-loads-nothing` | 1 | address table lookup with zero writable and zero readonly indexes | view_parse: ParseError |
| `v0-program-from-lookup` | 1 | program_id_index points at a looked-up address | view_sanitize: SanitizeError |
| `v1-13-signatures` | 1 | 13 signatures > max 12 | runtime_static: SanitizeFailure |
| `v1-65-addresses` | 1 | 65 addresses > max 64 | runtime_static: SanitizeFailure |
| `v1-65-instructions` | 1 | 65 instructions > max 64 | runtime_static: SanitizeFailure |
| `v1-duplicate-address` | 1 | address 1 duplicates address 0 | runtime_resolved: AccountLoadedTwice |
| `v1-extra-signature` | 1 | one more signature than num_required_signatures (trailing 64 bytes) | view_parse: ParseError |
| `v1-header-exceeds-addresses` | 1 | required_signatures + readonly_unsigned > num_addresses | view_sanitize: SanitizeError |
| `v1-heap-above-max` | 1 | heap 257 KiB > 256 KiB | runtime_static: SanitizeFailure |
| `v1-heap-below-min` | 1 | heap 31 KiB < 32 KiB | runtime_static: SanitizeFailure |
| `v1-heap-not-1k-multiple` | 1 | heap 32769 not a multiple of 1024 | runtime_static: SanitizeFailure |
| `v1-heap-zero` | 1 | heap bit set with value 0 | runtime_static: SanitizeFailure |
| `v1-ix-account-index-oob` | 1 | instruction account index == num_addresses | view_sanitize: SanitizeError |
| `v1-mask-unknown-bit` | 1 | config mask bit 5 set (reserved) | view_parse: SanitizeError |
| `v1-no-writable-signer` | 1 | num_readonly_signed == num_required_signatures (fee payer would be read-only) | view_sanitize: SanitizeError |
| `v1-program-index-oob` | 1 | program_id_index == num_addresses | view_sanitize: SanitizeError |
| `v1-program-is-fee-payer` | 1 | program_id_index = 0 (fee payer cannot be a program) | view_sanitize: SanitizeError |
| `v1-size-4097` | 1 | 4097 bytes > max 4096 | runtime_static: SanitizeFailure |
| `v1-trailing-byte` | 1 | one extra byte after the last signature | view_parse: ParseError |
| `v1-zero-signatures` | 1 | num_required_signatures = 0 | view_sanitize: SanitizeError |

## @solana/web3.js@3.1.1

### ACCEPTS-INVALID: 58 vectors in 28 classes

| class | n | example | detail |
|---|---|---|---|
| `legacy-duplicate-address` | 1 | duplicate static address in a legacy tx (static sanitize passes; runtime rejects) | runtime_resolved: AccountLoadedTwice |
| `legacy-program-is-fee-payer` | 1 | program_id_index = 0 | view_sanitize: SanitizeError |
| `legacy-size-over-1232` | 1 | legacy transaction of 1235 bytes (> 1232) | view_sanitize: SanitizeError |
| `mut-bitflip-legacy` | 1 | legacy template, byte 259 bit 4 flipped | view_sanitize: SanitizeError |
| `mut-bitflip-v0` | 2 | v0 template, byte 854 bit 7 flipped | view_sanitize: SanitizeError |
| `mut-bitflip-v1` | 7 | v1 template, byte 158 bit 7 flipped | view_sanitize: SanitizeError |
| `mut-bitflip-v1-config` | 9 | v1-config template, byte 157 bit 1 flipped | runtime_static: SanitizeFailure |
| `mut-bitflip-v1-header` | 16 | v1-config template, byte 1 bit 1 flipped | view_parse: ParseError |
| `mut-truncate-legacy` | 1 | legacy template cut to 805 of 806 bytes | view_parse: ParseError |
| `v0-lookup-loads-nothing` | 1 | address table lookup with zero writable and zero readonly indexes | view_parse: ParseError |
| `v0-program-from-lookup` | 1 | program_id_index points at a looked-up address | view_sanitize: SanitizeError |
| `v1-13-signatures` | 1 | 13 signatures > max 12 | runtime_static: SanitizeFailure |
| `v1-65-addresses` | 1 | 65 addresses > max 64 | runtime_static: SanitizeFailure |
| `v1-65-instructions` | 1 | 65 instructions > max 64 | runtime_static: SanitizeFailure |
| `v1-duplicate-address` | 1 | address 1 duplicates address 0 | runtime_resolved: AccountLoadedTwice |
| `v1-extra-signature` | 1 | one more signature than num_required_signatures (trailing 64 bytes) | view_parse: ParseError |
| `v1-header-exceeds-addresses` | 1 | required_signatures + readonly_unsigned > num_addresses | view_sanitize: SanitizeError |
| `v1-heap-above-max` | 1 | heap 257 KiB > 256 KiB | runtime_static: SanitizeFailure |
| `v1-heap-below-min` | 1 | heap 31 KiB < 32 KiB | runtime_static: SanitizeFailure |
| `v1-heap-not-1k-multiple` | 1 | heap 32769 not a multiple of 1024 | runtime_static: SanitizeFailure |
| `v1-heap-zero` | 1 | heap bit set with value 0 | runtime_static: SanitizeFailure |
| `v1-ix-account-index-oob` | 1 | instruction account index == num_addresses | view_sanitize: SanitizeError |
| `v1-no-writable-signer` | 1 | num_readonly_signed == num_required_signatures (fee payer would be read-only) | view_sanitize: SanitizeError |
| `v1-program-index-oob` | 1 | program_id_index == num_addresses | view_sanitize: SanitizeError |
| `v1-program-is-fee-payer` | 1 | program_id_index = 0 (fee payer cannot be a program) | view_sanitize: SanitizeError |
| `v1-size-4097` | 1 | 4097 bytes > max 4096 | runtime_static: SanitizeFailure |
| `v1-trailing-byte` | 1 | one extra byte after the last signature | view_parse: ParseError |
| `v1-zero-signatures` | 1 | num_required_signatures = 0 | view_sanitize: SanitizeError |

## @solana/web3.js@1.99.0

### ACCEPTS-INVALID: 48 vectors in 26 classes

| class | n | example | detail |
|---|---|---|---|
| `legacy-duplicate-address` | 1 | duplicate static address in a legacy tx (static sanitize passes; runtime rejects) | runtime_resolved: AccountLoadedTwice |
| `legacy-program-is-fee-payer` | 1 | program_id_index = 0 | view_sanitize: SanitizeError |
| `legacy-size-over-1232` | 1 | legacy transaction of 1235 bytes (> 1232) | view_sanitize: SanitizeError |
| `mut-bitflip-legacy` | 2 | legacy template, byte 259 bit 4 flipped | view_sanitize: SanitizeError |
| `mut-bitflip-v0` | 2 | v0 template, byte 854 bit 7 flipped | view_sanitize: SanitizeError |
| `mut-bitflip-v1-config` | 8 | v1-config template, byte 157 bit 1 flipped | runtime_static: SanitizeFailure |
| `mut-bitflip-v1-header` | 14 | v1-config template, byte 2 bit 1 flipped | view_sanitize: SanitizeError |
| `mut-truncate-legacy` | 1 | legacy template cut to 805 of 806 bytes | view_parse: ParseError |
| `v0-lookup-loads-nothing` | 1 | address table lookup with zero writable and zero readonly indexes | view_parse: ParseError |
| `v0-program-from-lookup` | 1 | program_id_index points at a looked-up address | view_sanitize: SanitizeError |
| `v1-13-signatures` | 1 | 13 signatures > max 12 | runtime_static: SanitizeFailure |
| `v1-65-addresses` | 1 | 65 addresses > max 64 | runtime_static: SanitizeFailure |
| `v1-65-instructions` | 1 | 65 instructions > max 64 | runtime_static: SanitizeFailure |
| `v1-duplicate-address` | 1 | address 1 duplicates address 0 | runtime_resolved: AccountLoadedTwice |
| `v1-header-exceeds-addresses` | 1 | required_signatures + readonly_unsigned > num_addresses | view_sanitize: SanitizeError |
| `v1-heap-above-max` | 1 | heap 257 KiB > 256 KiB | runtime_static: SanitizeFailure |
| `v1-heap-below-min` | 1 | heap 31 KiB < 32 KiB | runtime_static: SanitizeFailure |
| `v1-heap-not-1k-multiple` | 1 | heap 32769 not a multiple of 1024 | runtime_static: SanitizeFailure |
| `v1-heap-zero` | 1 | heap bit set with value 0 | runtime_static: SanitizeFailure |
| `v1-ix-account-index-oob` | 1 | instruction account index == num_addresses | view_sanitize: SanitizeError |
| `v1-message-in-legacy-envelope` | 1 | signatures first, then a 0x81 message: not a valid v1 or legacy/v0 transaction | view_parse: ParseError |
| `v1-no-writable-signer` | 1 | num_readonly_signed == num_required_signatures (fee payer would be read-only) | view_sanitize: SanitizeError |
| `v1-program-index-oob` | 1 | program_id_index == num_addresses | view_sanitize: SanitizeError |
| `v1-program-is-fee-payer` | 1 | program_id_index = 0 (fee payer cannot be a program) | view_sanitize: SanitizeError |
| `v1-size-4097` | 1 | 4097 bytes > max 4096 | runtime_static: SanitizeFailure |
| `v1-zero-signatures` | 1 | num_required_signatures = 0 | view_sanitize: SanitizeError |

### REJECTS-VALID: 327 vectors in 4 classes

| class | n | example | detail |
|---|---|---|---|
| `mut-bitflip-v1` | 228 | v1 template, byte 301 bit 2 flipped | Expected u64 value to be within the safe integer range |
| `v1-config-max-values` | 1 | u64::MAX priority fee lamports, u32::MAX limits, 256 KiB heap: format-valid (SDKs must not | Expected u64 value to be within the safe integer range |
| `v1-priority-fee-only` | 1 | only bits 0,1 set; value is 8-byte LE u64 total lamports | Expected u64 value to be within the safe integer range |
| `v1-random` | 97 | random well-formed v1 transaction | Expected u64 value to be within the safe integer range |

### ROUNDTRIP: 658 vectors in 13 classes

| class | n | example | detail |
|---|---|---|---|
| `mut-bitflip-v1-config` | 239 | v1-config template, byte 347 bit 6 flipped | ENCODE_ERROR: Serialization of version 1 transaction messages is not supported |
| `mut-bitflip-v1-header` | 306 | v1-config template, byte 2 bit 0 flipped | ENCODE_ERROR: Serialization of version 1 transaction messages is not supported |
| `v1-at-limits` | 1 | 12 signatures, 64 addresses, 64 instructions: all at the maximum | ENCODE_ERROR: Serialization of version 1 transaction messages is not supported |
| `v1-computebudget-ix-ignored` | 1 | ComputeBudget instructions present: valid no-ops; effective CU limit is 50000 and fee is 5 | ENCODE_ERROR: Serialization of version 1 transaction messages is not supported |
| `v1-empty-config` | 1 | mask = 0, no config values; defaults apply (CU limit 0!) | ENCODE_ERROR: Serialization of version 1 transaction messages is not supported |
| `v1-full-config` | 1 | all four config fields present (mask 0b11111) | ENCODE_ERROR: Serialization of version 1 transaction messages is not supported |
| `v1-heap-max` | 1 | heap exactly 256 KiB | ENCODE_ERROR: Serialization of version 1 transaction messages is not supported |
| `v1-heap-min` | 1 | heap exactly 32 KiB | ENCODE_ERROR: Serialization of version 1 transaction messages is not supported |
| `v1-ix-255-accounts` | 1 | one instruction with 255 account indexes | ENCODE_ERROR: Serialization of version 1 transaction messages is not supported |
| `v1-random` | 103 | random well-formed v1 transaction | ENCODE_ERROR: Serialization of version 1 transaction messages is not supported |
| `v1-size-4096` | 1 | exactly 4096 bytes (max) | ENCODE_ERROR: Serialization of version 1 transaction messages is not supported |
| `v1-size-over-1232` | 1 | v1 transaction larger than legacy PACKET_DATA_SIZE | ENCODE_ERROR: Serialization of version 1 transaction messages is not supported |
| `v1-zero-instructions` | 1 | num_instructions = 0 | ENCODE_ERROR: Serialization of version 1 transaction messages is not supported |

## solders@0.29.0

### ACCEPTS-INVALID: 7 vectors in 7 classes

| class | n | example | detail |
|---|---|---|---|
| `legacy-duplicate-address` | 1 | duplicate static address in a legacy tx (static sanitize passes; runtime rejects) | runtime_resolved: AccountLoadedTwice |
| `legacy-size-over-1232` | 1 | legacy transaction of 1235 bytes (> 1232) | view_sanitize: SanitizeError |
| `mut-bitflip-v1-config` | 1 | v1-config template, byte 163 bit 2 flipped | view_parse: ParseError |
| `mut-bitflip-v1-header` | 1 | v1-config template, byte 40 bit 1 flipped | view_parse: ParseError |
| `v1-extra-signature` | 1 | one more signature than num_required_signatures (trailing 64 bytes) | view_parse: ParseError |
| `v1-size-4097` | 1 | 4097 bytes > max 4096 | runtime_static: SanitizeFailure |
| `v1-trailing-byte` | 1 | one extra byte after the last signature | view_parse: ParseError |

## solana-go/v2@v2.2.0

### ACCEPTS-INVALID: 3 vectors in 3 classes

| class | n | example | detail |
|---|---|---|---|
| `legacy-duplicate-address` | 1 | duplicate static address in a legacy tx (static sanitize passes; runtime rejects) | runtime_resolved: AccountLoadedTwice |
| `legacy-size-over-1232` | 1 | legacy transaction of 1235 bytes (> 1232) | view_sanitize: SanitizeError |
| `v1-size-4097` | 1 | 4097 bytes > max 4096 | runtime_static: SanitizeFailure |

## gagliardetto/solana-go@v1.25.0

### ACCEPTS-INVALID: 3 vectors in 3 classes

| class | n | example | detail |
|---|---|---|---|
| `legacy-duplicate-address` | 1 | duplicate static address in a legacy tx (static sanitize passes; runtime rejects) | runtime_resolved: AccountLoadedTwice |
| `legacy-size-over-1232` | 1 | legacy transaction of 1235 bytes (> 1232) | view_sanitize: SanitizeError |
| `v1-size-4097` | 1 | 4097 bytes > max 4096 | runtime_static: SanitizeFailure |

## solana-primitives@0.3.0

### ACCEPTS-INVALID: 1 vectors in 1 classes

| class | n | example | detail |
|---|---|---|---|
| `legacy-duplicate-address` | 1 | duplicate static address in a legacy tx (static sanitize passes; runtime rejects) | runtime_resolved: AccountLoadedTwice |

