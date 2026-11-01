// Harness: solana-go v2 (github.com/solana-foundation/solana-go/v2).
package main

import (
	"bufio"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"runtime/debug"

	solana "github.com/solana-foundation/solana-go/v2"
)

type vec struct {
	ID    string `json:"id"`
	TxHex string `json:"tx_hex"`
}
type ix struct {
	ProgramIDIndex uint8   `json:"program_id_index"`
	Accounts       []int   `json:"accounts"`
	DataHex        string  `json:"data_hex"`
}
type cfg struct {
	PriorityFee *string `json:"priority_fee_lamports"`
	CU          *uint32 `json:"compute_unit_limit"`
	Loaded      *uint32 `json:"loaded_accounts_data_size_limit"`
	Heap        *uint32 `json:"heap_size"`
}
type fields struct {
	Version   string   `json:"version"`
	NumSigs   int      `json:"num_signatures"`
	Header    [3]uint8 `json:"header"`
	Keys      []string `json:"account_keys"`
	Blockhash string   `json:"recent_blockhash"`
	Ixs       []ix     `json:"instructions"`
	NumLookup int      `json:"num_address_table_lookups"`
	Config    *cfg     `json:"config"`
}
type res struct {
	ID       string  `json:"id"`
	Harness  string  `json:"harness"`
	Decode   bool    `json:"decode"`
	Error    *string `json:"error"`
	Sanitize *bool   `json:"sanitize"`
	Fields   *fields `json:"fields"`
	Re       *string `json:"reserialized_hex"`
}

func sp(s string) *string { return &s }

func harnessName() string {
	if bi, ok := debug.ReadBuildInfo(); ok {
		for _, d := range bi.Deps {
			if d.Path == "github.com/solana-foundation/solana-go/v2" {
				return "solana-go/v2@" + d.Version
			}
		}
	}
	return "solana-go/v2"
}

func main() {
	in, _ := os.Open(os.Args[1])
	out, _ := os.Create(os.Args[2])
	defer out.Close()
	sc := bufio.NewScanner(in)
	sc.Buffer(make([]byte, 1<<20), 1<<24)
	h := harnessName()
	for sc.Scan() {
		if len(sc.Bytes()) == 0 {
			continue
		}
		var v vec
		json.Unmarshal(sc.Bytes(), &v)
		b, _ := hex.DecodeString(v.TxHex)
		r := res{ID: v.ID, Harness: h}
		func() {
			defer func() {
				if p := recover(); p != nil {
					r.Error = sp(fmt.Sprintf("PANIC: %v", p))
				}
			}()
			tx, err := solana.TransactionFromBytes(b)
			if err != nil {
				r.Error = sp(err.Error())
				return
			}
			r.Decode = true
			serr := tx.Sanitize()
			ok := serr == nil
			r.Sanitize = &ok
			if serr != nil {
				r.Error = sp("sanitize: " + serr.Error())
			}
			m := tx.Message
			f := fields{NumSigs: len(tx.Signatures), Blockhash: m.RecentBlockhash.String()}
			f.Header = [3]uint8{m.Header.NumRequiredSignatures, m.Header.NumReadonlySignedAccounts, m.Header.NumReadonlyUnsignedAccounts}
			switch m.GetVersion() {
			case solana.MessageVersionV1:
				f.Version = "1"
			case solana.MessageVersionV0:
				f.Version = "0"
			default:
				f.Version = "legacy"
			}
			for _, k := range m.AccountKeys {
				f.Keys = append(f.Keys, k.String())
			}
			f.Ixs = []ix{}
			for _, ci := range m.Instructions {
				accs := []int{}
				for _, a := range ci.Accounts {
					accs = append(accs, int(a))
				}
				f.Ixs = append(f.Ixs, ix{ProgramIDIndex: uint8(ci.ProgramIDIndex), Accounts: accs, DataHex: hex.EncodeToString(ci.Data)})
			}
			f.Keys = append([]string{}, f.Keys...)
			if f.Keys == nil {
				f.Keys = []string{}
			}
			f.NumLookup = len(m.AddressTableLookups)
			if f.Version == "1" {
				c := &cfg{}
				tc := m.TransactionConfig
				if tc.PriorityFee != nil {
					c.PriorityFee = sp(fmt.Sprintf("%d", *tc.PriorityFee))
				}
				c.CU = tc.ComputeUnitLimit
				c.Loaded = tc.LoadedAccountsDataSizeLimit
				c.Heap = tc.HeapSize
				f.Config = c
			}
			r.Fields = &f
			re, err := tx.MarshalBinary()
			if err != nil {
				r.Re = sp("ENCODE_ERROR: " + err.Error())
			} else {
				r.Re = sp(hex.EncodeToString(re))
			}
		}()
		j, _ := json.Marshal(r)
		out.Write(j)
		out.Write([]byte("\n"))
	}
}
