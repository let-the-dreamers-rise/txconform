// Harness: @solana/kit (getTransactionDecoder + getCompiledTransactionMessageDecoder)
const k = require('@solana/kit');
const { b58, readVectors, errStr, result } = require('./common');
const H = '@solana/kit@' + JSON.parse(require('fs').readFileSync(require.resolve('@solana/kit').replace(/dist.*$/,'package.json'),'utf8')).version;

function configFrom(cm) {
  if (cm.version !== 1) return null;
  const c = { priority_fee_lamports: null, compute_unit_limit: null, loaded_accounts_data_size_limit: null, heap_size: null };
  const vals = [...cm.configValues];
  const mask = cm.configMask;
  if (mask & k.TRANSACTION_CONFIG_PRIORITY_FEE_LAMPORTS_BIT_MASK) c.priority_fee_lamports = vals.shift().value.toString();
  if (mask & k.TRANSACTION_CONFIG_COMPUTE_UNIT_LIMIT_BIT_MASK) c.compute_unit_limit = Number(vals.shift().value);
  if (mask & k.TRANSACTION_CONFIG_LOADED_ACCOUNTS_DATA_SIZE_LIMIT_BIT_MASK) c.loaded_accounts_data_size_limit = Number(vals.shift().value);
  if (mask & k.TRANSACTION_CONFIG_HEAP_SIZE_BIT_MASK) c.heap_size = Number(vals.shift().value);
  return c;
}

(async () => {
  const [vectors, out] = process.argv.slice(2);
  const fs = require('fs');
  const o = fs.createWriteStream(out);
  for await (const v of readVectors(vectors)) {
    const bytes = Uint8Array.from(Buffer.from(v.tx_hex, 'hex'));
    let line;
    try {
      const tx = k.getTransactionDecoder().decode(bytes);
      const cm = k.getCompiledTransactionMessageDecoder().decode(tx.messageBytes);
      let instructions;
      if (cm.version === 1) {
        instructions = cm.instructionHeaders.map((h, i) => ({
          program_id_index: h.programAccountIndex,
          accounts: [...(cm.instructionPayloads[i].instructionAccountIndices || [])],
          data_hex: Buffer.from(cm.instructionPayloads[i].instructionData || []).toString('hex'),
        }));
      } else {
        instructions = (cm.instructions || []).map((ix) => ({
          program_id_index: ix.programAddressIndex,
          accounts: [...(ix.accountIndices || [])],
          data_hex: Buffer.from(ix.data || []).toString('hex'),
        }));
      }
      const fields = {
        version: cm.version === 'legacy' ? 'legacy' : String(cm.version),
        num_signatures: Object.keys(tx.signatures).length,
        header: [cm.header.numSignerAccounts, cm.header.numReadonlySignerAccounts, cm.header.numReadonlyNonSignerAccounts],
        account_keys: cm.staticAccounts.map(String),
        recent_blockhash: String(cm.lifetimeToken),
        instructions,
        num_address_table_lookups: (cm.addressTableLookups || []).length,
        config: configFrom(cm),
      };
      // deep round trip: re-encode the compiled message and the transaction
      let re = null;
      try {
        const msg = k.getCompiledTransactionMessageEncoder().encode(cm);
        const tx2 = { ...tx, messageBytes: msg };
        re = Buffer.from(k.getTransactionEncoder().encode(tx2)).toString('hex');
      } catch (e) { re = 'ENCODE_ERROR: ' + errStr(e); }
      line = result(v.id, H, { decode: true, sanitize: null, fields, reserialized_hex: re });
    } catch (e) {
      line = result(v.id, H, { decode: false, error: errStr(e) });
    }
    o.write(line + '\n');
  }
  o.end();
})();
