/// Benchmark comparing `cargo rustdoc` (sequential) vs `cargo doc` (parallel).
///
/// Run with:
/// ```bash
/// cargo run --example benchmark-cargo-doc
/// ```
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Cargo.toml".to_string());

    let packages: Vec<String> = std::env::args().skip(2).collect();

    if packages.is_empty() {
        eprintln!("Usage: benchmark-cargo-doc <manifest-path> <pkg1> <pkg2> ...");
        eprintln!();
        eprintln!("Example:");
        eprintln!("  cargo run --example benchmark-cargo-doc -- Cargo.toml rustdoc-json public-api rustup-toolchain");
        std::process::exit(1);
    }

    let toolchain = std::env::var("BENCHMARK_TOOLCHAIN")
        .unwrap_or_else(|_| "nightly".to_string());

    println!("Benchmarking rustdoc JSON generation for {} packages", packages.len());
    println!("Packages: {:?}", packages);
    println!("Manifest: {manifest_path}");
    println!("Toolchain: {toolchain}");
    println!();

    // --- Benchmark 1: Sequential cargo rustdoc (one per package) ---
    println!("=== Sequential: cargo rustdoc (one package at a time) ===");
    let start = Instant::now();
    for pkg in &packages {
        let target_dir = tempfile::tempdir()?;
        let path = rustdoc_json::Builder::default()
            .toolchain(&toolchain)
            .manifest_path(&manifest_path)
            .package(pkg)
            .target_dir(&target_dir)
            .quiet(true)
            .build()?;
        println!("  Built {pkg}: {path:?}");
    }
    let sequential_elapsed = start.elapsed();
    println!("Sequential total: {sequential_elapsed:.2?}");
    println!();

    // --- Benchmark 2: Parallel cargo doc (all packages at once) ---
    println!("=== Parallel: cargo doc (all packages at once) ===");
    let target_dir = tempfile::tempdir()?;
    let start = Instant::now();
    let paths = rustdoc_json::Builder::default()
        .toolchain(&toolchain)
        .manifest_path(&manifest_path)
        .packages(&packages)
        .target_dir(&target_dir)
        .quiet(true)
        .build_using_cargo_doc()?;
    for path in &paths {
        println!("  Built: {path:?}");
    }
    let parallel_elapsed = start.elapsed();
    println!("Parallel total: {parallel_elapsed:.2?}");
    println!();

    // --- Summary ---
    println!("=== Summary ===");
    println!("Sequential (cargo rustdoc): {sequential_elapsed:.2?}");
    println!("Parallel   (cargo doc):     {parallel_elapsed:.2?}");
    let speedup = sequential_elapsed.as_secs_f64() / parallel_elapsed.as_secs_f64();
    println!("Speedup: {speedup:.2}x");

    Ok(())
}
