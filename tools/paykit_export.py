#!/usr/bin/env python3
"""Export txconform's Agave-rejected vectors as pay-kit conformance vectors.

pay-kit (solana-foundation/pay-kit, harness/) drives every SDK over JSON vectors with the
shape {id, intent, mode, description, spec, bugType, tier, role, provenance, input, expect}
(harness/src/conformance/schema.ts). In `verify-transaction` mode the runner receives
`input.transaction` (base64 wire bytes) and must answer `outcome` and `rejectCode`.

Every vector exported here is one that Agave will not accept at the packet or runtime layer,
so any payment verifier must answer `reject` with `invalid-payload` (pay-kit ruling T4 maps
oversize and malformed version-1 messages to that code). Valid txconform vectors are not
exported: pay-kit would reject them for payment reasons (amount, recipient), which is correct
and says nothing about wire conformance.

This is a proposed contribution format; the pay-kit maintainers own the final shape.
"""
import base64, json, sys

vectors_path, oracle_path, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
vectors = {json.loads(l)["id"]: json.loads(l) for l in open(vectors_path) if l.strip()}
oracle = {json.loads(l)["id"]: json.loads(l) for l in open(oracle_path) if l.strip()}

def agave_layer(o):
    for k in ("view_parse", "view_sanitize", "runtime_static", "runtime_resolved"):
        v = o[k]
        if isinstance(v, dict) and "Err" in v:
            return k, v["Err"]
    return None, None

out = []
for vid, o in oracle.items():
    if o["valid"]:
        continue
    v = vectors[vid]
    if v.get("loaded_writable_hex") or v.get("loaded_readonly_hex"):
        continue  # needs lookup-table state a fixture RPC would have to serve; skip for now
    layer, err = agave_layer(o)
    out.append({
        "id": f"txconform-{vid}",
        "intent": "charge",
        "mode": "verify-transaction",
        "description": f"txconform {v['class']}: {v['note']} (Agave {layer}: {err})",
        "spec": {"doc": "SIMD-0385" if v["spec"].startswith("SIMD") else "solana-transaction-wire-format",
                 "section": v["class"], "quote": v["note"]},
        "bugType": "wire",
        "tier": "offline",
        "role": "server",
        "provenance": "generated",
        "input": {"transaction": base64.b64encode(bytes.fromhex(v["tx_hex"])).decode()},
        "expect": {"outcome": "reject", "rejectCode": "invalid-payload"},
    })
json.dump(out, open(out_path, "w"), indent=1)
print(f"exported {len(out)} pay-kit verify-transaction vectors to {out_path}")
