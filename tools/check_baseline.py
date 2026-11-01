#!/usr/bin/env python3
"""Fail CI when a harness shows a divergence class that is not in the accepted baseline.

baseline.json maps harness name -> category -> list of vector classes that are known
(reported upstream, or by-design) and therefore do not fail the build. Anything new
fails, so an SDK release that regresses is caught by the weekly run.
"""
import json, sys, collections

details = json.load(open(sys.argv[1]))
baseline = json.load(open(sys.argv[2])) if len(sys.argv) > 2 else {}
vectors = {json.loads(l)["id"]: json.loads(l) for l in open("vectors/vectors.jsonl") if l.strip()}
new = []
for harness, cats in details.items():
    hkey = harness.rsplit("@", 1)[0]
    for cat, items in cats.items():
        known = set(baseline.get(hkey, {}).get(cat, []))
        classes = {vectors[vid]["class"] for vid, _ in items}
        for c in sorted(classes - known):
            new.append(f"{harness}: {cat}: {c}")
if new:
    print("NEW DIVERGENCES (not in baseline.json):")
    print("\n".join(new))
    sys.exit(1)
print("no new divergences beyond baseline")
