//! Build script for cargo-optimize

fn main() {
    // Set version information
    println!(
        "cargo:rustc-env=CARGO_OPTIMIZE_BUILD_DATE={}",
        std::env::var("SOURCE_DATE_EPOCH").unwrap_or_else(|_| "unknown".to_string())
    );

    // Note: Self-optimization has been disabled to avoid cyclic dependencies.
    // Users of this crate can still use it to optimize their own builds!
}
