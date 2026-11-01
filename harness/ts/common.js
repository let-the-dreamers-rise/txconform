// Shared helpers for the TypeScript/JavaScript harnesses.
const fs = require('fs');
const readline = require('readline');

function b58(bytes) {
  // minimal base58 (bitcoin alphabet) to avoid depending on the SDK under test
  const ALPH = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
  let x = BigInt('0x' + Buffer.from(bytes).toString('hex').padStart(2, '0'));
  let out = '';
  while (x > 0n) { out = ALPH[Number(x % 58n)] + out; x /= 58n; }
  for (const b of bytes) { if (b === 0) out = '1' + out; else break; }
  return out || '1';
}

async function* readVectors(path) {
  const rl = readline.createInterface({ input: fs.createReadStream(path) });
  for await (const line of rl) { if (line.trim()) yield JSON.parse(line); }
}

function errStr(e) {
  const m = (e && (e.message || String(e))) || 'error';
  return m.split('\n')[0].slice(0, 300);
}

function result(id, harness, obj) {
  return JSON.stringify({ id, harness, decode: false, error: null, sanitize: null, fields: null, reserialized_hex: null, ...obj });
}

module.exports = { b58, readVectors, errStr, result };
