#!/usr/bin/env python3
"""RPC-path harness: what does a real node's JSON-RPC say about each vector?

Sends every vector to `simulateTransaction` on a running `solana-test-validator`
(encoding base64, sigVerify false, replaceRecentBlockhash true) and classifies the answer:

  format accepted   the node deserialized and sanitized the transaction. The simulation itself
                    may still fail (AccountNotFound, program missing); that is not a format
                    verdict, so `decode` is true and `fields` is null (the RPC does not echo
                    the parsed message).
  format rejected   JSON-RPC error -32602 / "failed to deserialize" / "invalid transaction"
                    or a sanitize failure: `decode` false, `error` carries the node's text.
  skipped           v0 vectors whose lookup tables do not exist on the local ledger; the node
                    cannot resolve them, which says nothing about the wire format.

Usage: rpc_harness.py <vectors.jsonl> <out.jsonl> [rpc_url]
"""
import base64, json, sys, time, urllib.request

vectors_path, out_path = sys.argv[1], sys.argv[2]
url = sys.argv[3] if len(sys.argv) > 3 else "http://127.0.0.1:8899"

def rpc(method, params, timeout=20):
    body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode()
    req = urllib.request.Request(url, data=body, headers={"content-type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read())

# wait for the validator
for _ in range(120):
    try:
        if rpc("getHealth", [])["result"] == "ok":
            break
    except Exception:
        time.sleep(1)
else:
    sys.exit("validator not healthy")
ver = rpc("getVersion", [])["result"]
harness = f"agave-rpc@{ver.get('solana-core', '?')}"

FORMAT_REJECT_MARKERS = ("failed to deserialize", "invalid transaction", "sanitize", "Transaction too large",
                         "too many", "failed to sanitize", "invalid base64", "Transaction version")

with open(out_path, "w") as out:
    for line in open(vectors_path):
        if not line.strip():
            continue
        v = json.loads(line)
        rec = {"id": v["id"], "harness": harness, "decode": False, "error": None, "sanitize": None,
               "fields": None, "fields_unavailable": True, "reserialized_hex": None}
        if v.get("loaded_writable_hex") or v.get("loaded_readonly_hex"):
            rec["skip"] = True
            rec["error"] = "v0 lookup tables not present on local ledger"
            out.write(json.dumps(rec) + "\n")
            continue
        b64 = base64.b64encode(bytes.fromhex(v["tx_hex"])).decode()
        try:
            resp = rpc("simulateTransaction", [b64, {"encoding": "base64", "sigVerify": False,
                                                     "replaceRecentBlockhash": True, "commitment": "processed"}])
        except Exception as e:  # transport-level failure (e.g. 413 for oversize bodies is a format reject)
            rec["error"] = f"transport: {e}"
            out.write(json.dumps(rec) + "\n")
            continue
        if "error" in resp:
            msg = str(resp["error"].get("message", ""))
            rec["error"] = f"rpc {resp['error'].get('code')}: {msg}"
            # -32602 invalid params = the node could not deserialize/sanitize the transaction
            if resp["error"].get("code") == -32602 or any(m.lower() in msg.lower() for m in FORMAT_REJECT_MARKERS):
                rec["decode"] = False
            else:
                # some other node-side error: format was parsed. Keep as accept but note it.
                rec["decode"] = True
                rec["sanitize"] = True
        else:
            rec["decode"] = True
            rec["sanitize"] = True
            err = resp["result"]["value"].get("err")
            rec["error"] = None if err is None else f"simulation: {json.dumps(err)[:160]}"
            # Transaction-level load errors are the runtime's sanitize verdict surfacing late,
            # not account-state failures: count them as rejects of the transaction.
            if err in ("AccountLoadedTwice", "TooManyAccountLocks", "SanitizeFailure", "DuplicateInstruction"):
                rec["sanitize"] = False
        out.write(json.dumps(rec) + "\n")
print("done", harness)
