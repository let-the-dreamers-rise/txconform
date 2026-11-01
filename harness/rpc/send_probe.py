#!/usr/bin/env python3
"""Send probe: does a transaction the RPC accepts actually land?

The packet-level parser (`agave-transaction-view`) rejects trailing bytes and an extra
signature; `simulateTransaction` (which deserializes with the SDK) accepts them. This probe
measures what happens end to end on a local validator: sign a real transfer from an
airdropped keypair, send the clean bytes and the mutated bytes with `sendTransaction`, and
poll `getSignatureStatuses` to see which ones land.

Usage: send_probe.py <out.json> [rpc_url]
"""
import base64, json, os, sys, time, urllib.request
from solders.keypair import Keypair
from solders.hash import Hash
from solders.message import Message, MessageV1, TransactionConfig
from solders.transaction import VersionedTransaction
from solders.system_program import transfer, TransferParams
from solders.pubkey import Pubkey

out_path = sys.argv[1]
url = sys.argv[2] if len(sys.argv) > 2 else "http://127.0.0.1:8899"

def rpc(method, params, timeout=30):
    body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode()
    req = urllib.request.Request(url, data=body, headers={"content-type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read())

for _ in range(120):
    try:
        if rpc("getHealth", [])["result"] == "ok":
            break
    except Exception:
        time.sleep(1)
version = rpc("getVersion", [])["result"].get("solana-core", "?")

payer = Keypair()
dest = Pubkey.new_unique()
sig = rpc("requestAirdrop", [str(payer.pubkey()), 5_000_000_000, {"commitment": "confirmed"}])["result"]
balance = 0
for _ in range(120):
    balance = rpc("getBalance", [str(payer.pubkey()), {"commitment": "confirmed"}])["result"]["value"]
    if balance > 0:
        break
    time.sleep(0.5)
assert balance > 0, f"airdrop failed: signature {sig}, balance still 0 after 60 s"

def compact_u16(n):
    out = bytearray()
    while True:
        b = n & 0x7F
        n >>= 7
        if n:
            out.append(b | 0x80)
        else:
            out.append(b)
            return bytes(out)

def blockhash():
    return Hash.from_string(rpc("getLatestBlockhash", [{"commitment": "confirmed"}])["result"]["value"]["blockhash"])

def build(kind, lamports):
    ix = transfer(TransferParams(from_pubkey=payer.pubkey(), to_pubkey=dest, lamports=lamports))
    bh = blockhash()
    if kind == "v1":
        msg = MessageV1.try_compile(payer.pubkey(), [ix], bh, TransactionConfig(compute_unit_limit=50_000, loaded_accounts_data_size_limit=1_000_000))
        # VersionedTransaction signs over the prefixed message. Note for anyone assembling by
        # hand: in solders `bytes(msg)` omits the version byte for v0 and v1 (documented); the
        # signed bytes are `to_bytes_versioned(msg)`.
        return bytes(VersionedTransaction(msg, [payer]))
    msg = Message.new_with_blockhash([ix], payer.pubkey(), bh)
    mb = bytes(msg)
    s = bytes(payer.sign_message(mb))
    return compact_u16(1) + s + mb  # legacy: count || signatures || message

cases = []
lam = 100_000_000  # above the rent-exempt minimum for a new system account
for kind in ("v1", "legacy"):
    for variant in ("clean", "trailing-byte", "extra-signature"):
        for skip_preflight in (False, True):
            lam += 1  # distinct amount so each transaction has a distinct signature/message
            b = build(kind, lam)
            if variant == "trailing-byte":
                b = b + b"\x00"
            elif variant == "extra-signature":
                b = b + os.urandom(64) if kind == "v1" else compact_u16(2) + b[1:] + os.urandom(64)
            case = {"kind": kind, "variant": variant, "skip_preflight": skip_preflight, "len": len(b)}
            try:
                resp = rpc("sendTransaction", [base64.b64encode(b).decode(),
                                               {"encoding": "base64", "skipPreflight": skip_preflight,
                                                "preflightCommitment": "processed", "maxRetries": 0}])
            except Exception as e:
                case["send"] = f"transport: {e}"
                cases.append(case)
                continue
            if "error" in resp:
                case["send"] = f"rpc error {resp['error'].get('code')}: {str(resp['error'].get('message'))[:200]}"
                case["landed"] = None
            else:
                txsig = resp["result"]
                case["send"] = f"accepted, signature {txsig[:12]}..."
                landed = None
                for _ in range(40):  # ~20 s
                    st = rpc("getSignatureStatuses", [[txsig], {"searchTransactionHistory": True}])["result"]["value"][0]
                    if st is not None:
                        landed = {"slot": st.get("slot"), "err": st.get("err"), "status": st.get("confirmationStatus")}
                        break
                    time.sleep(0.5)
                case["landed"] = landed if landed else False
            cases.append(case)

result = {"validator": f"agave {version}", "cases": cases}
json.dump(result, open(out_path, "w"), indent=1)
print(json.dumps(result, indent=1))
