#!/usr/bin/env python3
"""txconform diff: compare SDK harness results against the Agave oracle.

Categories (most to least severe):
  ACCEPTS-INVALID : the SDK decodes (and, where it has one, sanitizes) a transaction
                    that Agave will not accept. Risky for wallets, co-signers,
                    paymasters and facilitators that approve what they decode.
  REJECTS-VALID   : the SDK fails on a transaction Agave accepts (breaks reading
                    blocks/indexing, or stops a valid tx from being signed).
  FIELD-MISMATCH  : both accept, but a parsed field differs from Agave's view.
  ROUNDTRIP       : both accept, but decode->encode does not reproduce the input bytes.
"""
import json, sys, collections, argparse

def load(path):
    return {r["id"]: r for r in (json.loads(l) for l in open(path) if l.strip())}

# SIMD-0385 defaults for unset config fields. An SDK may report "absent" or the
# default value; both mean the same thing to the runtime, so compare effective values.
DEFAULTS = {"priority_fee_lamports": "0", "compute_unit_limit": 0, "loaded_accounts_data_size_limit": 0, "heap_size": 32 * 1024}

def norm_fields(f):
    if f is None:
        return None
    g = dict(f)
    if g.get("version") == "1":
        c = dict(g.get("config") or {})
        for k, d in DEFAULTS.items():
            if c.get(k) is None:
                c[k] = d
        c["priority_fee_lamports"] = str(c["priority_fee_lamports"])
        g["config"] = c
    else:
        g["config"] = None
    return g

def field_diffs(a, b):
    out = []
    for k in ["version", "num_signatures", "header", "account_keys", "recent_blockhash", "num_address_table_lookups", "config"]:
        if a.get(k) != b.get(k):
            out.append(f"{k}: oracle={json.dumps(a.get(k))[:80]} sdk={json.dumps(b.get(k))[:80]}")
    ia, ib = a.get("instructions") or [], b.get("instructions") or []
    if len(ia) != len(ib):
        out.append(f"instructions: count oracle={len(ia)} sdk={len(ib)}")
    else:
        for i, (x, y) in enumerate(zip(ia, ib)):
            if x != y:
                out.append(f"instructions[{i}]: oracle={json.dumps(x)[:80]} sdk={json.dumps(y)[:80]}")
                break
    return out

def agave_reason(o):
    for k in ["view_parse", "view_sanitize", "runtime_static", "runtime_resolved", "sdk_decode", "sdk_sanitize"]:
        v = o[k]
        if isinstance(v, dict) and "Err" in v:
            return f"{k}: {v['Err']}"
    return "?"

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--oracle", default="vectors/oracle.jsonl")
    ap.add_argument("--vectors", default="vectors/vectors.jsonl")
    ap.add_argument("--results", nargs="+", required=True)
    ap.add_argument("--report", default="results/REPORT.md")
    ap.add_argument("--details", default="results/details.json")
    a = ap.parse_args()
    oracle = load(a.oracle)
    vectors = load(a.vectors)
    report = ["# txconform report", "", f"Vectors: {len(oracle)} (oracle = Agave: agave-transaction-view + solana-runtime-transaction + solana-transaction)", ""]
    details = {}
    summary_rows = []
    for path in a.results:
        res = load(path)
        hname = next(iter(res.values()))["harness"]
        cats = collections.defaultdict(list)
        for vid, o in oracle.items():
            r = res.get(vid)
            if r is None:
                cats["MISSING"].append((vid, "no result"))
                continue
            if r.get("skip"):
                continue  # the harness could not judge this vector (e.g. RPC path without lookup tables)
            sdk_ok = r["decode"] and (r["sanitize"] is not False)
            if o["valid"] and not sdk_ok:
                cats["REJECTS-VALID"].append((vid, r.get("error")))
            elif (not o["valid"]) and sdk_ok:
                cats["ACCEPTS-INVALID"].append((vid, agave_reason(o)))
            elif o["valid"] and sdk_ok:
                fd = field_diffs(norm_fields(o["fields"]), norm_fields(r["fields"])) if r["fields"] else ["fields: none produced"]
                if fd:
                    cats["FIELD-MISMATCH"].append((vid, "; ".join(fd)))
                re = r.get("reserialized_hex")
                if re is not None and re != vectors[vid]["tx_hex"]:
                    cats["ROUNDTRIP"].append((vid, (re[:60] + "...") if not re.startswith("ENCODE_ERROR") else re))
        n_valid = sum(1 for o in oracle.values() if o["valid"])
        n_invalid = len(oracle) - n_valid
        summary_rows.append((hname, n_valid, n_invalid, len(cats["REJECTS-VALID"]), len(cats["ACCEPTS-INVALID"]), len(cats["FIELD-MISMATCH"]), len(cats["ROUNDTRIP"])))
        details[hname] = {k: v for k, v in cats.items()}
        report.append(f"## {hname}")
        report.append("")
        for cat in ["ACCEPTS-INVALID", "REJECTS-VALID", "FIELD-MISMATCH", "ROUNDTRIP", "MISSING"]:
            items = cats.get(cat, [])
            if not items:
                continue
            # group by class
            byclass = collections.defaultdict(list)
            for vid, why in items:
                byclass[vectors[vid]["class"]].append((vid, why))
            report.append(f"### {cat}: {len(items)} vectors in {len(byclass)} classes")
            report.append("")
            report.append("| class | n | example | detail |")
            report.append("|---|---|---|---|")
            for cls, its in sorted(byclass.items()):
                vid, why = its[0]
                note = vectors[vid]["note"].replace("|", "/")
                report.append(f"| `{cls}` | {len(its)} | {note[:90]} | {str(why).replace('|','/')[:160]} |")
            report.append("")
        if not any(cats.values()):
            report.append("No divergences.")
            report.append("")
    report.insert(4, "| harness | valid vectors | invalid vectors | rejects-valid | accepts-invalid | field-mismatch | roundtrip |")
    report.insert(5, "|---|---|---|---|---|---|---|")
    for i, row in enumerate(summary_rows):
        report.insert(6 + i, "| " + " | ".join(str(x) for x in row) + " |")
    report.insert(6 + len(summary_rows), "")
    open(a.report, "w").write("\n".join(report) + "\n")
    json.dump(details, open(a.details, "w"), indent=1)
    print("\n".join(report))

if __name__ == "__main__":
    main()
