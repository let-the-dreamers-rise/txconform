mod judge;
mod wire;

use clap::{Parser, Subcommand};
use std::io::{BufRead, Write};

#[derive(Parser)]
#[command(name = "txconform-oracle", about = "Spec-derived Solana transaction vectors, judged by Agave's own parser")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Generate vectors (JSONL) from the specification alone.
    Gen {
        #[arg(long, default_value_t = 20261101)]
        seed: u64,
        #[arg(long, default_value_t = 200)]
        random: usize,
        #[arg(long)]
        out: String,
    },
    /// Judge vectors with Agave; writes JSONL verdicts.
    Judge {
        #[arg(long)]
        vectors: String,
        #[arg(long)]
        out: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Gen { seed, random, out } => {
            let mut g = wire::Gen::new(seed);
            g.generate(random);
            let mut f = std::fs::File::create(&out).expect("create");
            for v in &g.out {
                writeln!(f, "{}", serde_json::to_string(v).unwrap()).unwrap();
            }
            eprintln!("wrote {} vectors to {}", g.out.len(), out);
        }
        Cmd::Judge { vectors, out } => {
            let f = std::fs::File::open(&vectors).expect("open vectors");
            let mut o = std::fs::File::create(&out).expect("create out");
            let mut n = 0;
            let mut disagree = 0;
            for line in std::io::BufReader::new(f).lines() {
                let line = line.unwrap();
                if line.trim().is_empty() {
                    continue;
                }
                let v: wire::Vector = serde_json::from_str(&line).expect("vector json");
                let verdict = judge::judge(&v);
                if verdict.valid != v.expect_valid {
                    disagree += 1;
                    eprintln!(
                        "SPEC/AGAVE DISAGREEMENT {} ({}): spec says valid={} agave says valid={} view={:?}/{:?} runtime={:?}/{:?} sdk={:?}/{:?}",
                        v.id, v.note, v.expect_valid, verdict.valid, verdict.view_parse, verdict.view_sanitize,
                        verdict.runtime_static, verdict.runtime_resolved, verdict.sdk_decode, verdict.sdk_sanitize
                    );
                }
                writeln!(o, "{}", serde_json::to_string(&verdict).unwrap()).unwrap();
                n += 1;
            }
            eprintln!("judged {n} vectors, {disagree} spec/agave disagreements -> {out}");
        }
    }
}
