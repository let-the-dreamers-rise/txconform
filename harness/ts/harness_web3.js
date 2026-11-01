// Harness: @solana/web3.js (3.x by default; pass "web3js1" as 3rd arg for 1.x)
const [vectors, out, pkg] = process.argv.slice(2);
const w3 = require(pkg || '@solana/web3.js');
const H = (pkg === 'web3js1' ? '@solana/web3.js@' : '@solana/web3.js@') + JSON.parse(require('fs').readFileSync(require.resolve(pkg || '@solana/web3.js').replace(/(lib|dist).*$/,'package.json'),'utf8')).version;
const { readVectors, errStr, result } = require('./common');

(async () => {
  const fs = require('fs');
  const o = fs.createWriteStream(out);
  for await (const v of readVectors(vectors)) {
    const bytes = Uint8Array.from(Buffer.from(v.tx_hex, 'hex'));
    let line;
    try {
      const tx = w3.VersionedTransaction.deserialize(bytes);
      const m = tx.message;
      const cfg = m.transactionConfig;
      const fields = {
        version: m.version === 'legacy' ? 'legacy' : String(m.version),
        num_signatures: tx.signatures.length,
        header: [m.header.numRequiredSignatures, m.header.numReadonlySignedAccounts, m.header.numReadonlyUnsignedAccounts],
        account_keys: m.staticAccountKeys.map((k) => k.toBase58()),
        recent_blockhash: m.recentBlockhash,
        instructions: m.compiledInstructions.map((ix) => ({
          program_id_index: ix.programIdIndex,
          accounts: [...ix.accountKeyIndexes],
          data_hex: Buffer.from(ix.data).toString('hex'),
        })),
        num_address_table_lookups: (m.addressTableLookups || []).length,
        config: m.version === 1 ? {
          priority_fee_lamports: cfg && (cfg.priorityFeeLamports ?? cfg.priorityFee) != null ? (cfg.priorityFeeLamports ?? cfg.priorityFee).toString() : null, // 1.x names it priorityFee
          compute_unit_limit: cfg && cfg.computeUnitLimit != null ? Number(cfg.computeUnitLimit) : null,
          loaded_accounts_data_size_limit: cfg && cfg.loadedAccountsDataSizeLimit != null ? Number(cfg.loadedAccountsDataSizeLimit) : null,
          heap_size: cfg && cfg.heapSize != null ? Number(cfg.heapSize) : null,
        } : null,
      };
      let re = null;
      try { re = Buffer.from(tx.serialize()).toString('hex'); } catch (e) { re = 'ENCODE_ERROR: ' + errStr(e); }
      line = result(v.id, H, { decode: true, sanitize: null, fields, reserialized_hex: re });
    } catch (e) {
      line = result(v.id, H, { decode: false, error: errStr(e) });
    }
    o.write(line + '\n');
  }
  o.end();
})();
