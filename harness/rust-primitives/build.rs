fn main() {
    // Expose the resolved solana-primitives version as a compile-time constant.
    let lock = std::fs::read_to_string("Cargo.lock").unwrap_or_default();
    let mut v = "unknown".to_string();
    if let Some(i) = lock.find("name = \"solana-primitives\"") {
        if let Some(j) = lock[i..].find("version = \"") {
            let s = &lock[i + j + 11..];
            v = s[..s.find('"').unwrap_or(0)].to_string();
        }
    }
    println!("cargo:rustc-env=CARGO_PKG_VERSION_PRIMITIVES={v}");
    println!("cargo:rerun-if-changed=Cargo.lock");
}
