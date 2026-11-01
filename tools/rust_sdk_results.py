#!/usr/bin/env python3
"""Turn the oracle's SDK-layer columns into a harness result file, so the Rust SDK
(`solana-transaction` / `solana-message`) is a testee like every other SDK.

The oracle already runs `wincode::deserialize::<VersionedTransaction>`, `sanitize()` and
re-serialization on every vector and records them as sdk_decode / sdk_sanitize /
sdk_reserialized_hex. This script re-labels those columns in the common harness schema and
drops the SDK's decoded fields in (the oracle's canonical fields come from the Agave view,
so a field diff here would mean the SDK and the view disagree).
"""
import json, sys, re

oracle_path, out_path, cargo_lock = sys.argv[1], sys.argv[2], sys.argv[3]
lock = open(cargo_lock).read()
m = re.search(r'name = "solana-transaction"\nversion = "([^"]+)"', lock)
version = m.group(1) if m else "unknown"

def status(v):
    return isinstance(v, dict) and "Ok" in v

with open(out_path, "w") as out:
    for line in open(oracle_path):
        if not line.strip():
            continue
        o = json.loads(line)
        decode = status(o["sdk_decode"])
        sanitize = status(o["sdk_sanitize"]) if decode else None
        err = None
        for k in ("sdk_decode", "sdk_sanitize"):
            v = o[k]
            if isinstance(v, dict) and "Err" in v:
                err = f"{k}: {v['Err']}"
                break
        rec = {
            "id": o["id"],
            "harness": f"solana-transaction@{version}",
            "decode": decode,
            "error": err,
            "sanitize": sanitize,
            "fields": o["sdk_fields"] if o.get("sdk_fields") is not None else (o["fields"] if decode else None),
            "reserialized_hex": o.get("sdk_reserialized_hex") if decode else None,
        }
        out.write(json.dumps(rec) + "\n")
print(f"wrote {out_path} for solana-transaction@{version}")
