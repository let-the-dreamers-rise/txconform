"""Harness: solders (Python). Decodes with VersionedTransaction.from_bytes, then sanitize()."""
import json, sys, importlib.metadata
from solders.transaction import VersionedTransaction
from solders.message import MessageV1, MessageV0

H = "solders@" + importlib.metadata.version("solders")

def fields_of(tx):
    m = tx.message
    if isinstance(m, MessageV1):
        version = "1"
    elif isinstance(m, MessageV0):
        version = "0"
    else:
        version = "legacy"
    cfg = None
    if version == "1":
        c = m.config
        cfg = {
            "priority_fee_lamports": None if c.priority_fee is None else str(c.priority_fee),
            "compute_unit_limit": c.compute_unit_limit,
            "loaded_accounts_data_size_limit": c.loaded_accounts_data_size_limit,
            "heap_size": c.heap_size,
        }
    lookups = len(m.address_table_lookups) if version == "0" else 0
    return {
        "version": version,
        "num_signatures": len(tx.signatures),
        "header": [m.header.num_required_signatures, m.header.num_readonly_signed_accounts, m.header.num_readonly_unsigned_accounts],
        "account_keys": [str(k) for k in m.account_keys],
        "recent_blockhash": str(m.recent_blockhash),
        "instructions": [
            {"program_id_index": ix.program_id_index, "accounts": list(ix.accounts), "data_hex": bytes(ix.data).hex()}
            for ix in m.instructions
        ],
        "num_address_table_lookups": lookups,
        "config": cfg,
    }

def main(vectors, out):
    with open(vectors) as f, open(out, "w") as o:
        for line in f:
            if not line.strip():
                continue
            v = json.loads(line)
            b = bytes.fromhex(v["tx_hex"])
            r = {"id": v["id"], "harness": H, "decode": False, "error": None, "sanitize": None, "fields": None, "reserialized_hex": None}
            try:
                tx = VersionedTransaction.from_bytes(b)
                r["decode"] = True
                try:
                    tx.sanitize()
                    r["sanitize"] = True
                except Exception as e:  # noqa
                    r["sanitize"] = False
                    r["error"] = f"sanitize: {e}"[:300]
                try:
                    r["fields"] = fields_of(tx)
                except Exception as e:  # noqa
                    r["error"] = (r["error"] or "") + f" fields: {e}"[:300]
                try:
                    r["reserialized_hex"] = bytes(tx).hex()
                except Exception as e:  # noqa
                    r["reserialized_hex"] = f"ENCODE_ERROR: {e}"[:300]
            except Exception as e:  # noqa
                r["error"] = str(e).split("\n")[0][:300]
            o.write(json.dumps(r) + "\n")

if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
